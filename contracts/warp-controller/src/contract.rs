use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
    Uint64,
};
use cw_utils::{must_pay, nonpayable};

use crate::{
    execute, migrate, query, reply,
    state::{CONFIG, STATE},
    ContractError,
};

use controller::{Config, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, State};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        current_job_id: Uint64::one(),
        q: Uint64::zero(),
    };

    let config = Config {
        owner: deps
            .api
            .addr_validate(&msg.owner.unwrap_or_else(|| info.sender.to_string()))?,
        fee_denom: msg.fee_denom,
        fee_collector: deps
            .api
            .addr_validate(&msg.fee_collector.unwrap_or_else(|| info.sender.to_string()))?,
        warp_account_code_id: msg.warp_account_code_id,
        minimum_reward: msg.minimum_reward,
        creation_fee_percentage: msg.creation_fee,
        cancellation_fee_percentage: msg.cancellation_fee,
        resolver_address: deps.api.addr_validate(&msg.resolver_address)?,
        job_account_tracker_address: deps.api.addr_validate(&msg.job_account_tracker_address)?,
        t_max: msg.t_max,
        t_min: msg.t_min,
        a_max: msg.a_max,
        a_min: msg.a_min,
        q_max: msg.q_max,
        creation_fee_min: msg.creation_fee_min,
        creation_fee_max: msg.creation_fee_max,
        burn_fee_min: msg.burn_fee_min,
        maintenance_fee_min: msg.maintenance_fee_min,
        maintenance_fee_max: msg.maintenance_fee_max,
        duration_days_left: msg.duration_days_left,
        duration_days_right: msg.duration_days_right,
        queue_size_left: msg.queue_size_left,
        queue_size_right: msg.queue_size_right,
        burn_fee_rate: msg.burn_fee_rate,
    };

    if config.a_max < config.a_min {
        return Err(ContractError::MaxFeeUnderMinFee {});
    }

    if config.t_max < config.t_min {
        return Err(ContractError::MaxTimeUnderMinTime {});
    }

    if config.minimum_reward < config.a_min {
        return Err(ContractError::RewardSmallerThanFee {});
    }

    if config.creation_fee_percentage.u64() > 100 {
        return Err(ContractError::CreationFeeTooHigh {});
    }

    if config.cancellation_fee_percentage.u64() > 100 {
        return Err(ContractError::CancellationFeeTooHigh {});
    }

    STATE.save(deps.storage, &state)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    match msg {
        ExecuteMsg::CreateJob(data) => {
            // IBC denoms can be passed alongside native, can't use must_pay
            let fee_denom_paid_amount = info
                .funds
                .iter()
                .find(|f| f.denom == config.fee_denom)
                .unwrap()
                .amount;

            execute::job::create_job(deps, env, info, *data, config, fee_denom_paid_amount)
        }
        ExecuteMsg::DeleteJob(data) => {
            let fee_denom_paid_amount = must_pay(&info, &config.fee_denom).unwrap();
            execute::job::delete_job(deps, env, info, *data, config, fee_denom_paid_amount)
        }
        ExecuteMsg::UpdateJob(data) => execute::job::update_job(deps, env, info, *data),
        ExecuteMsg::ExecuteJob(data) => {
            nonpayable(&info).unwrap();
            execute::job::execute_job(deps, env, info, *data, config)
        }
        ExecuteMsg::EvictJob(data) => {
            nonpayable(&info).unwrap();
            execute::job::evict_job(deps, env, info, *data, config)
        }
        ExecuteMsg::UpdateConfig(data) => {
            nonpayable(&info).unwrap();
            execute::controller::update_config(deps, env, info, *data, config)
        }
        ExecuteMsg::MigrateLegacyAccounts(data) => {
            nonpayable(&info).unwrap();
            migrate::legacy_account::migrate_legacy_accounts(deps, info, *data, config)
        }
        ExecuteMsg::MigrateFreeJobAccounts(data) => {
            nonpayable(&info).unwrap();
            migrate::job_account::migrate_free_job_accounts(deps.as_ref(), env, info, *data, config)
        }
        ExecuteMsg::MigrateTakenJobAccounts(data) => {
            nonpayable(&info).unwrap();
            migrate::job_account::migrate_taken_job_accounts(
                deps.as_ref(),
                env,
                info,
                *data,
                config,
            )
        }

        ExecuteMsg::MigratePendingJobs(data) => {
            nonpayable(&info).unwrap();
            migrate::job::migrate_pending_jobs(deps, env, info, *data)
        }
        ExecuteMsg::MigrateFinishedJobs(data) => {
            nonpayable(&info).unwrap();
            migrate::job::migrate_finished_jobs(deps, env, info, *data)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryJob(data) => to_json_binary(&query::job::query_job(deps, env, data)?),
        QueryMsg::QueryJobs(data) => to_json_binary(&query::job::query_jobs(deps, env, data)?),

        // For job account, please query it via the account tracker contract
        QueryMsg::QueryLegacyAccount(data) => {
            to_json_binary(&query::account::query_legacy_account(deps, env, data)?)
        }
        QueryMsg::QueryLegacyAccounts(data) => {
            to_json_binary(&query::account::query_legacy_accounts(deps, env, data)?)
        }

        QueryMsg::QueryConfig(data) => {
            to_json_binary(&query::controller::query_config(deps, env, data)?)
        }
        QueryMsg::QueryState(data) => to_json_binary(&query::controller::query_state(deps, env, data)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    config.warp_account_code_id = msg.warp_account_code_id;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    match msg.id {
        // use 0 as hack to call create_job_account_and_job
        0 => reply::account::create_job_account_and_job(deps, env, msg, config),
        _id => reply::job::execute_job(deps, env, msg, config),
    }
}

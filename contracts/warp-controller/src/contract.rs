use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult, Uint128, Uint64,
};
use cw_storage_plus::Item;
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
        QueryMsg::QueryJob(data) => to_binary(&query::job::query_job(deps, env, data)?),
        QueryMsg::QueryJobs(data) => to_binary(&query::job::query_jobs(deps, env, data)?),

        // For job account, please query it via the account tracker contract
        QueryMsg::QueryLegacyAccount(data) => {
            to_binary(&query::account::query_legacy_account(deps, env, data)?)
        }
        QueryMsg::QueryLegacyAccounts(data) => {
            to_binary(&query::account::query_legacy_accounts(deps, env, data)?)
        }

        QueryMsg::QueryConfig(data) => {
            to_binary(&query::controller::query_config(deps, env, data)?)
        }
        QueryMsg::QueryState(data) => to_binary(&query::controller::query_state(deps, env, data)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    //CONFIG
    #[cw_serde]
    pub struct V1Config {
        pub owner: Addr,
        pub fee_denom: String,
        pub fee_collector: Addr,
        pub warp_account_code_id: Uint64,
        pub minimum_reward: Uint128,
        pub creation_fee_percentage: Uint64,
        pub cancellation_fee_percentage: Uint64,
        // maximum time for evictions
        pub t_max: Uint64,
        // minimum time for evictions
        pub t_min: Uint64,
        // maximum fee for evictions
        pub a_max: Uint128,
        // minimum fee for evictions
        pub a_min: Uint128,
        // maximum length of queue modifier for evictions
        pub q_max: Uint64,
        pub creation_fee_min: Uint128,
        pub creation_fee_max: Uint128,
        pub burn_fee_min: Uint128,
        pub maintenance_fee_min: Uint128,
        pub maintenance_fee_max: Uint128,
        // duration_days fn interval [left, right]
        pub duration_days_left: Uint128,
        pub duration_days_right: Uint128,
        // queue_size fn interval [left, right]
        pub queue_size_left: Uint128,
        pub queue_size_right: Uint128,
        pub burn_fee_rate: Uint128,
    }

    const V1CONFIG: Item<V1Config> = Item::new("config");

    let v1_config = V1CONFIG.load(deps.storage)?;

    CONFIG.save(
        deps.storage,
        &Config {
            owner: v1_config.owner,
            fee_denom: v1_config.fee_denom,
            fee_collector: v1_config.fee_collector,
            warp_account_code_id: msg.warp_account_code_id,
            minimum_reward: v1_config.minimum_reward,
            creation_fee_percentage: v1_config.creation_fee_percentage,
            cancellation_fee_percentage: v1_config.cancellation_fee_percentage,
            resolver_address: deps.api.addr_validate(&msg.resolver_address)?,
            job_account_tracker_address: deps
                .api
                .addr_validate(&msg.job_account_tracker_address)?,
            t_max: v1_config.t_max,
            t_min: v1_config.t_min,
            a_max: v1_config.a_max,
            a_min: v1_config.a_min,
            q_max: v1_config.q_max,
            creation_fee_min: v1_config.creation_fee_min,
            creation_fee_max: v1_config.creation_fee_max,
            burn_fee_min: v1_config.burn_fee_min,
            maintenance_fee_min: v1_config.maintenance_fee_min,
            maintenance_fee_max: v1_config.maintenance_fee_max,
            duration_days_left: v1_config.duration_days_left,
            duration_days_right: v1_config.duration_days_right,
            queue_size_left: v1_config.queue_size_left,
            queue_size_right: v1_config.queue_size_right,
            burn_fee_rate: v1_config.burn_fee_rate,
        },
    )?;

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

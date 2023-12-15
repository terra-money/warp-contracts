use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response,
    StdResult, SubMsg, Uint64,
};
use cw_utils::nonpayable;

use crate::{
    execute, migrate, query, reply,
    state::{CONFIG, STATE},
    util::msg::build_instantiate_account_tracker_msg,
    ContractError,
};

use controller::{Config, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, State};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
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
        resolver_address: deps.api.addr_validate(&msg.resolver_address)?,
        // placeholder, will be updated in reply
        account_tracker_address: deps.api.addr_validate(&msg.resolver_address)?,
        creation_fee_min: msg.creation_fee_min,
        creation_fee_max: msg.creation_fee_max,
        burn_fee_min: msg.burn_fee_min,
        maintenance_fee_min: msg.maintenance_fee_min,
        maintenance_fee_max: msg.maintenance_fee_max,
        duration_days_min: msg.duration_days_min,
        duration_days_max: msg.duration_days_max,
        queue_size_left: msg.queue_size_left,
        queue_size_right: msg.queue_size_right,
        burn_fee_rate: msg.burn_fee_rate,
        cancellation_fee_rate: msg.cancellation_fee_rate,
    };

    if config.creation_fee_max < config.creation_fee_min {
        return Err(ContractError::CreationMaxFeeUnderMinFee {});
    }

    if config.maintenance_fee_max < config.maintenance_fee_min {
        return Err(ContractError::MaintenanceMaxFeeUnderMinFee {});
    }

    if config.duration_days_max < config.duration_days_min {
        return Err(ContractError::DurationMaxDaysUnderMinDays {});
    }

    if config.cancellation_fee_rate.u64() > 100 {
        return Err(ContractError::CancellationFeeTooHigh {});
    }

    if config.burn_fee_rate.u128() > 100 {
        return Err(ContractError::BurnFeeTooHigh {});
    }

    STATE.save(deps.storage, &state)?;
    CONFIG.save(deps.storage, &config)?;

    let submsgs = vec![SubMsg {
        id: REPLY_ID_INSTANTIATE_SUB_CONTRACTS,
        msg: build_instantiate_account_tracker_msg(
            config.owner.to_string(),
            env.contract.address.to_string(),
            msg.account_tracker_code_id.u64(),
        ),
        gas_limit: None,
        reply_on: ReplyOn::Always,
    }];

    Ok(Response::new().add_submessages(submsgs))
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
        ExecuteMsg::CreateJob(data) => execute::job::create_job(deps, env, info, data, config),
        ExecuteMsg::DeleteJob(data) => execute::job::delete_job(deps, env, info, data, config),
        ExecuteMsg::UpdateJob(data) => execute::job::update_job(deps, env, info, data),
        ExecuteMsg::ExecuteJob(data) => {
            nonpayable(&info).unwrap();
            execute::job::execute_job(deps, env, info, data, config)
        }
        ExecuteMsg::EvictJob(data) => {
            nonpayable(&info).unwrap();
            execute::job::evict_job(deps, env, info, data, config)
        }
        ExecuteMsg::UpdateConfig(data) => {
            nonpayable(&info).unwrap();
            execute::controller::update_config(deps, env, info, data, config)
        }
        ExecuteMsg::MigrateAccounts(data) => {
            nonpayable(&info).unwrap();
            migrate::account::migrate_accounts(deps.as_ref(), env, info, data, config)
        }

        ExecuteMsg::MigratePendingJobs(data) => {
            nonpayable(&info).unwrap();
            migrate::job::migrate_pending_jobs(deps, env, info, data)
        }
        ExecuteMsg::MigrateFinishedJobs(data) => {
            nonpayable(&info).unwrap();
            migrate::job::migrate_finished_jobs(deps, env, info, data)
        }

        ExecuteMsg::CreateFundingAccount(data) => {
            execute::account::create_funding_account(deps, env, info, data)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryJob(data) => to_binary(&query::job::query_job(deps, env, data)?),
        QueryMsg::QueryJobs(data) => to_binary(&query::job::query_jobs(deps, env, data)?),
        QueryMsg::QueryConfig(data) => {
            to_binary(&query::controller::query_config(deps, env, data)?)
        }
        QueryMsg::QueryState(data) => to_binary(&query::controller::query_state(deps, env, data)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub const REPLY_ID_CREATE_JOB_ACCOUNT_AND_JOB: u64 = 0;
pub const REPLY_ID_CREATE_FUNDING_ACCOUNT_AND_JOB: u64 = 1;
pub const REPLY_ID_CREATE_FUNDING_ACCOUNT: u64 = 2;
pub const REPLY_ID_INSTANTIATE_SUB_CONTRACTS: u64 = 3;
pub const REPLY_ID_EXECUTE_JOB: u64 = 4;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    match msg.id {
        REPLY_ID_CREATE_JOB_ACCOUNT_AND_JOB => {
            reply::account::create_account_and_job(deps, env, msg, config)
        }
        REPLY_ID_CREATE_FUNDING_ACCOUNT_AND_JOB => {
            reply::account::create_funding_account_and_job(deps, env, msg, config)
        }
        REPLY_ID_CREATE_FUNDING_ACCOUNT => {
            reply::account::create_funding_account(deps, env, msg, config)
        }
        REPLY_ID_INSTANTIATE_SUB_CONTRACTS => {
            reply::job::instantiate_sub_contracts(deps, env, msg, config)
        }
        REPLY_ID_EXECUTE_JOB => reply::job::execute_job(deps, env, msg, config),
        _ => panic!(),
    }
}

use crate::state::CONFIG;
use crate::{execute, query, reply, state::STATE, ContractError};

use controller::{Config, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, State};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
    Uint64,
};

pub const REPLY_ID_CREATE_ACCOUNT: u64 = 0;
pub const REPLY_ID_CREATE_JOB_ACCOUNT: u64 = 1;
pub const REPLY_ID_CREATE_ACCOUNT_AND_JOB: u64 = 2;
pub const REPLY_ID_CREATE_JOB_ACCOUNT_AND_JOB: u64 = 3;
pub const REPLY_ID_EXECUTE_JOB: u64 = 4;

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
        t_max: msg.t_max,
        t_min: msg.t_min,
        a_max: msg.a_max,
        a_min: msg.a_min,
        q_max: msg.q_max,
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
    match msg {
        ExecuteMsg::CreateJob(data) => execute::job::create_job(deps, env, info, data),
        ExecuteMsg::DeleteJob(data) => execute::job::delete_job(deps, env, info, data),
        ExecuteMsg::UpdateJob(data) => execute::job::update_job(deps, env, info, data),
        ExecuteMsg::ExecuteJob(data) => execute::job::execute_job(deps, env, info, data),
        ExecuteMsg::EvictJob(data) => execute::job::evict_job(deps, env, info, data),

        ExecuteMsg::CreateAccount(data) => execute::account::create_account(deps, env, info, data),

        ExecuteMsg::CreateAccountAndJob(data) => {
            execute::account::create_account_and_job(deps, env, info, data)
        }

        ExecuteMsg::UpdateConfig(data) => execute::controller::update_config(deps, env, info, data),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryJob(data) => to_binary(&query::job::query_job(deps, env, data)?),
        QueryMsg::QueryJobs(data) => to_binary(&query::job::query_jobs(deps, env, data)?),

        QueryMsg::QueryAccount(data) => to_binary(&query::account::query_account(deps, env, data)?),
        QueryMsg::QueryAccounts(data) => {
            to_binary(&query::account::query_accounts(deps, env, data)?)
        }

        QueryMsg::QueryJobAccount(data) => {
            to_binary(&query::account::query_job_account(deps, env, data)?)
        }

        QueryMsg::QueryConfig(data) => {
            to_binary(&query::controller::query_config(deps, env, data)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        REPLY_ID_CREATE_ACCOUNT => reply::account::create_account_and_job(
            deps,
            env,
            msg,
            false,
            false,
            "save_account".to_string(),
        ),
        REPLY_ID_CREATE_JOB_ACCOUNT => reply::account::create_account_and_job(
            deps,
            env,
            msg,
            false,
            true,
            "save_account".to_string(),
        ),
        REPLY_ID_CREATE_ACCOUNT_AND_JOB => reply::account::create_account_and_job(
            deps,
            env,
            msg,
            true,
            false,
            "save_account_and_job".to_string(),
        ),
        REPLY_ID_CREATE_JOB_ACCOUNT_AND_JOB => reply::account::create_account_and_job(
            deps,
            env,
            msg,
            true,
            true,
            "save_job_account_and_job".to_string(),
        ),
        REPLY_ID_EXECUTE_JOB => reply::job::execute_job(deps, env, msg),
        _ => Err(ContractError::UnknownReplyId {}),
    }
}

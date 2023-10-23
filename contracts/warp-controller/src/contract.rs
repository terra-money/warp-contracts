use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult, Uint128, Uint64,
};
use cw_storage_plus::Item;
use cw_utils::{must_pay, nonpayable};

use crate::state::CONFIG;
use crate::{execute, query, reply, state::STATE, ContractError};

use controller::{Config, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, State};

// Reply id for job creation
// From a totally new user using warp for the first time, does not have account yet, let alone sub account
// So we create both account and sub account and job
pub const REPLY_ID_CREATE_MAIN_ACCOUNT_AND_SUB_ACCOUNT_AND_JOB: u64 = 1;
// Reply id for job creation
// From an existing user, who has main account, but does not have available sub account
// So we create sub account and job
pub const REPLY_ID_CREATE_SUB_ACCOUNT_AND_JOB: u64 = 2;
// Reply id for job execution
pub const REPLY_ID_EXECUTE_JOB: u64 = 3;

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
    let config = CONFIG.load(deps.storage)?;
    match msg {
        ExecuteMsg::CreateJob(data) => {
            let fee_denom_paid_amount = must_pay(&info, &config.fee_denom).unwrap();
            execute::job::create_job(deps, env, info, data, config, fee_denom_paid_amount)
        }
        ExecuteMsg::DeleteJob(data) => {
            let fee_denom_paid_amount = must_pay(&info, &config.fee_denom).unwrap();
            execute::job::delete_job(deps, env, info, data, config, fee_denom_paid_amount)
        }
        ExecuteMsg::UpdateJob(data) => {
            let fee_denom_paid_amount = must_pay(&info, &config.fee_denom).unwrap();
            execute::job::update_job(deps, env, info, data, config, fee_denom_paid_amount)
        }
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
            execute::controller::update_config(deps, env, info, data)
        }

        ExecuteMsg::MigrateAccounts(data) => {
            nonpayable(&info).unwrap();
            execute::controller::migrate_accounts(deps, env, info, data)
        }
        ExecuteMsg::MigratePendingJobs(data) => {
            nonpayable(&info).unwrap();
            execute::controller::migrate_pending_jobs(deps, env, info, data)
        }
        ExecuteMsg::MigrateFinishedJobs(data) => {
            nonpayable(&info).unwrap();
            execute::controller::migrate_finished_jobs(deps, env, info, data)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryJob(data) => to_binary(&query::job::query_job(deps, env, data)?),
        QueryMsg::QueryJobs(data) => to_binary(&query::job::query_jobs(deps, env, data)?),

        QueryMsg::QueryMainAccount(data) => {
            to_binary(&query::account::query_main_account(deps, env, data)?)
        }
        QueryMsg::QueryMainAccounts(data) => {
            to_binary(&query::account::query_main_accounts(deps, env, data)?)
        }

        QueryMsg::QueryConfig(data) => {
            to_binary(&query::controller::query_config(deps, env, data)?)
        }
        QueryMsg::QueryState(data) => to_binary(&query::controller::query_state(deps, env, data)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    //STATE
    #[cw_serde]
    pub struct V1State {
        pub current_job_id: Uint64,
        pub current_template_id: Uint64,
        pub q: Uint64,
    }

    const V1STATE: Item<V1State> = Item::new("state");
    let v1_state = V1STATE.load(deps.storage)?;

    STATE.save(
        deps.storage,
        &State {
            current_job_id: v1_state.current_job_id,
            q: v1_state.q,
        },
    )?;

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
            t_max: v1_config.t_max,
            t_min: v1_config.t_min,
            a_max: v1_config.a_max,
            a_min: v1_config.a_min,
            q_max: v1_config.q_max,
        },
    )?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    match msg.id {
        // Main account has been created, now create sub account and job
        REPLY_ID_CREATE_MAIN_ACCOUNT_AND_SUB_ACCOUNT_AND_JOB => {
            reply::account::create_main_account_and_sub_account_and_job(deps, env, msg, config)
        }
        // Sub account has been created, now create job
        REPLY_ID_CREATE_SUB_ACCOUNT_AND_JOB => {
            reply::account::create_sub_account_and_job(deps, env, msg)
        }
        // Job has been executed
        REPLY_ID_EXECUTE_JOB => reply::job::execute_job(deps, env, msg, config),
        _ => Err(ContractError::UnknownReplyId {}),
    }
}

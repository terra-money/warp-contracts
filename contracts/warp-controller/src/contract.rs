use crate::reply;
use crate::state::{CONFIG, FINISHED_JOBS, PENDING_JOBS};
use crate::{execute, query, state::STATE, ContractError};
use controller::account::AssetInfo;
use controller::job::{Job, JobStatus};
use cosmwasm_schema::cw_serde;

use controller::{Config, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, State};
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Reply, Response,
    StdResult, Uint128, Uint64,
};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex, UniqueIndex};
use resolver::condition::Condition;
use resolver::variable::{
    ExternalExpr, ExternalVariable, QueryExpr, QueryVariable, StaticVariable, UpdateFn, Variable,
    VariableKind,
};

pub const REPLY_ID_CREATE_ACCOUNT: u64 = 0;
pub const REPLY_ID_CREATE_SUB_ACCOUNT: u64 = 1;
pub const REPLY_ID_CREATE_ACCOUNT_AND_JOB: u64 = 2;
pub const REPLY_ID_CREATE_SUB_ACCOUNT_AND_JOB: u64 = 3;
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
            to_binary(&query::account::query_account_used_by_job(deps, env, data)?)
        }

        QueryMsg::QueryConfig(data) => {
            to_binary(&query::controller::query_config(deps, env, data)?)
        }
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
            warp_account_code_id: v1_config.warp_account_code_id,
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

    //JOBS
    #[cw_serde]
    pub struct V1Job {
        pub id: Uint64,
        pub owner: Addr,
        pub last_update_time: Uint64,
        pub name: String,
        pub description: String,
        pub labels: Vec<String>,
        pub status: JobStatus,
        pub condition: Condition,
        pub msgs: Vec<String>,
        pub vars: Vec<V1Variable>,
        pub recurring: bool,
        pub requeue_on_evict: bool,
        pub reward: Uint128,
        pub assets_to_withdraw: Vec<AssetInfo>,
    }

    #[cw_serde]
    pub enum V1Variable {
        Static(V1StaticVariable),
        External(V1ExternalVariable),
        Query(V1QueryVariable),
    }

    #[cw_serde]
    pub struct V1StaticVariable {
        pub kind: VariableKind,
        pub name: String,
        pub value: String,
        pub update_fn: Option<UpdateFn>,
    }

    #[cw_serde]
    pub struct V1ExternalVariable {
        pub kind: VariableKind,
        pub name: String,
        pub init_fn: ExternalExpr,
        pub reinitialize: bool,
        pub value: Option<String>, //none if uninitialized
        pub update_fn: Option<UpdateFn>,
    }

    #[cw_serde]
    pub struct V1QueryVariable {
        pub kind: VariableKind,
        pub name: String,
        pub init_fn: QueryExpr,
        pub reinitialize: bool,
        pub value: Option<String>, //none if uninitialized
        pub update_fn: Option<UpdateFn>,
    }

    pub struct V1JobIndexes<'a> {
        pub reward: UniqueIndex<'a, (u128, u64), V1Job>,
        pub publish_time: MultiIndex<'a, u64, V1Job, u64>,
    }

    impl IndexList<V1Job> for V1JobIndexes<'_> {
        fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<V1Job>> + '_> {
            let v: Vec<&dyn Index<V1Job>> = vec![&self.reward, &self.publish_time];
            Box::new(v.into_iter())
        }
    }

    #[allow(non_snake_case)]
    pub fn V1_PENDING_JOBS<'a>() -> IndexedMap<'a, u64, V1Job, V1JobIndexes<'a>> {
        let indexes = V1JobIndexes {
            reward: UniqueIndex::new(
                |job| (job.reward.u128(), job.id.u64()),
                "pending_jobs__reward_v2",
            ),
            publish_time: MultiIndex::new(
                |_pk, job| job.last_update_time.u64(),
                "pending_jobs_v2",
                "pending_jobs__publish_timestamp_v2",
            ),
        };
        IndexedMap::new("pending_jobs_v2", indexes)
    }

    let job_keys: Result<Vec<_>, _> = V1_PENDING_JOBS()
        .keys(deps.storage, None, None, Order::Ascending)
        .collect();
    let job_keys = job_keys?;
    for job_key in job_keys {
        let v1_job = V1_PENDING_JOBS().load(deps.storage, job_key)?;
        let mut new_vars = vec![];
        for var in v1_job.vars {
            new_vars.push(match var {
                V1Variable::Static(v) => Variable::Static(StaticVariable {
                    kind: v.kind,
                    name: v.name,
                    encode: false,
                    value: v.value,
                    update_fn: v.update_fn,
                }),
                V1Variable::External(v) => Variable::External(ExternalVariable {
                    kind: v.kind,
                    name: v.name,
                    encode: false,
                    init_fn: v.init_fn,
                    reinitialize: v.reinitialize,
                    value: v.value,
                    update_fn: v.update_fn,
                }),
                V1Variable::Query(v) => Variable::Query(QueryVariable {
                    kind: v.kind,
                    name: v.name,
                    encode: false,
                    init_fn: v.init_fn,
                    reinitialize: v.reinitialize,
                    value: v.value,
                    update_fn: v.update_fn,
                }),
            })
        }

        let mut new_msgs = "[".to_string();

        for msg in v1_job.msgs {
            new_msgs.push_str(msg.as_str());
        }

        new_msgs.push(']');

        PENDING_JOBS().save(
            deps.storage,
            job_key,
            &Job {
                id: v1_job.id,
                prev_id: None,
                owner: v1_job.owner,
                account: None,
                last_update_time: v1_job.last_update_time,
                name: v1_job.name,
                description: v1_job.description,
                labels: v1_job.labels,
                status: v1_job.status,
                condition: serde_json_wasm::to_string(&v1_job.condition)?,
                terminate_condition: None,
                msgs: new_msgs.to_string(),
                vars: serde_json_wasm::to_string(&new_vars)?,
                recurring: v1_job.recurring,
                requeue_on_evict: v1_job.requeue_on_evict,
                reward: v1_job.reward,
                assets_to_withdraw: v1_job.assets_to_withdraw,
            },
        )?;
    }

    #[allow(non_snake_case)]
    pub fn V1_FINISHED_JOBS<'a>() -> IndexedMap<'a, u64, V1Job, V1JobIndexes<'a>> {
        let indexes = V1JobIndexes {
            reward: UniqueIndex::new(
                |job| (job.reward.u128(), job.id.u64()),
                "finished_jobs__reward_v2",
            ),
            publish_time: MultiIndex::new(
                |_pk, job| job.last_update_time.u64(),
                "finished_jobs_v2",
                "finished_jobs__publish_timestamp_v2",
            ),
        };
        IndexedMap::new("finished_jobs_v2", indexes)
    }

    let job_keys: Result<Vec<_>, _> = V1_FINISHED_JOBS()
        .keys(deps.storage, None, None, Order::Ascending)
        .collect();
    let job_keys = job_keys?;
    for job_key in job_keys {
        let v1_job = V1_FINISHED_JOBS().load(deps.storage, job_key)?;
        let mut new_vars = vec![];
        for var in v1_job.vars {
            new_vars.push(match var {
                V1Variable::Static(v) => Variable::Static(StaticVariable {
                    kind: v.kind,
                    name: v.name,
                    encode: false,
                    value: v.value,
                    update_fn: v.update_fn,
                }),
                V1Variable::External(v) => Variable::External(ExternalVariable {
                    kind: v.kind,
                    name: v.name,
                    encode: false,
                    init_fn: v.init_fn,
                    reinitialize: v.reinitialize,
                    value: v.value,
                    update_fn: v.update_fn,
                }),
                V1Variable::Query(v) => Variable::Query(QueryVariable {
                    kind: v.kind,
                    name: v.name,
                    encode: false,
                    init_fn: v.init_fn,
                    reinitialize: v.reinitialize,
                    value: v.value,
                    update_fn: v.update_fn,
                }),
            })
        }

        let mut new_msgs = "[".to_string();

        for msg in v1_job.msgs {
            new_msgs.push_str(msg.as_str());
        }

        new_msgs.push(']');

        FINISHED_JOBS().save(
            deps.storage,
            job_key,
            &Job {
                id: v1_job.id,
                prev_id: None,
                owner: v1_job.owner,
                account: None,
                last_update_time: v1_job.last_update_time,
                name: v1_job.name,
                description: v1_job.description,
                labels: v1_job.labels,
                status: v1_job.status,
                condition: serde_json_wasm::to_string(&v1_job.condition)?,
                terminate_condition: None,
                msgs: new_msgs,
                vars: serde_json_wasm::to_string(&new_vars)?,
                recurring: v1_job.recurring,
                requeue_on_evict: v1_job.requeue_on_evict,
                reward: v1_job.reward,
                assets_to_withdraw: v1_job.assets_to_withdraw,
            },
        )?;
    }

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
        REPLY_ID_CREATE_SUB_ACCOUNT => reply::account::create_account_and_job(
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
        REPLY_ID_CREATE_SUB_ACCOUNT_AND_JOB => reply::account::create_account_and_job(
            deps,
            env,
            msg,
            true,
            true,
            "save_sub_account_and_job".to_string(),
        ),
        REPLY_ID_EXECUTE_JOB => reply::job::execute_job(deps, env, msg),
        _ => Err(ContractError::UnknownReplyId {}),
    }
}

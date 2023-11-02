use crate::state::{ACCOUNTS, CONFIG, FINISHED_JOBS, PENDING_JOBS};
use crate::ContractError;
use controller::{MigrateAccountsMsg, MigrateJobsMsg, UpdateConfigMsg};
use cosmwasm_schema::cw_serde;

use controller::account::AssetInfo;
use controller::job::{Execution, Job, JobStatus};
use cosmwasm_std::{
    to_binary, Addr, DepsMut, Env, MessageInfo, Order, Response, StdError, Uint128, Uint64, WasmMsg,
};
use cw_storage_plus::{Bound, Index, IndexList, IndexedMap, MultiIndex, UniqueIndex};
use resolver::condition::StringValue;
use resolver::variable::{
    ExternalExpr, ExternalVariable, FnValue, QueryExpr, QueryVariable, StaticVariable, UpdateFn,
    Variable, VariableKind,
};

#[cw_serde]
pub struct V1Job {
    pub id: Uint64,
    pub owner: Addr,
    pub last_update_time: Uint64,
    pub name: String,
    pub description: String,
    pub labels: Vec<String>,
    pub status: JobStatus,
    pub terminate_condition: Option<String>,
    pub condition: String,
    pub msgs: String,
    pub vars: String,
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
    pub encode: bool,
}

#[cw_serde]
pub struct V1ExternalVariable {
    pub kind: VariableKind,
    pub name: String,
    pub encode: bool,
    pub init_fn: ExternalExpr,
    pub reinitialize: bool,
    pub value: Option<String>, //none if uninitialized
    pub update_fn: Option<UpdateFn>,
}

#[cw_serde]
pub struct V1QueryVariable {
    pub kind: VariableKind,
    pub name: String,
    pub encode: bool,
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

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    config.owner = match data.owner {
        None => config.owner,
        Some(data) => deps.api.addr_validate(data.as_str())?,
    };

    config.fee_collector = match data.fee_collector {
        None => config.fee_collector,
        Some(data) => deps.api.addr_validate(data.as_str())?,
    };
    config.minimum_reward = data.minimum_reward.unwrap_or(config.minimum_reward);
    config.creation_fee_percentage = data
        .creation_fee_percentage
        .unwrap_or(config.creation_fee_percentage);
    config.cancellation_fee_percentage = data
        .cancellation_fee_percentage
        .unwrap_or(config.cancellation_fee_percentage);

    config.a_max = data.a_max.unwrap_or(config.a_max);
    config.a_min = data.a_min.unwrap_or(config.a_min);
    config.t_max = data.t_max.unwrap_or(config.t_max);
    config.t_min = data.t_min.unwrap_or(config.t_min);
    config.q_max = data.q_max.unwrap_or(config.q_max);

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

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("config_owner", config.owner)
        .add_attribute("config_fee_collector", config.fee_collector)
        .add_attribute("config_minimum_reward", config.minimum_reward)
        .add_attribute(
            "config_creation_fee_percentage",
            config.creation_fee_percentage,
        )
        .add_attribute(
            "config_cancellation_fee_percentage",
            config.cancellation_fee_percentage,
        )
        .add_attribute("config_a_max", config.a_max)
        .add_attribute("config_a_min", config.a_min)
        .add_attribute("config_t_max", config.t_max)
        .add_attribute("config_t_min", config.t_min)
        .add_attribute("config_q_max", config.q_max))
}

pub fn migrate_accounts(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MigrateAccountsMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let start_after = match msg.start_after {
        None => None,
        Some(s) => Some(deps.api.addr_validate(s.as_str())?),
    };
    let start_after = start_after.map(Bound::exclusive);

    let account_keys: Result<Vec<_>, _> = ACCOUNTS()
        .keys(deps.storage, start_after, None, Order::Ascending)
        .take(msg.limit as usize)
        .collect();
    let account_keys = account_keys?;
    let mut migration_msgs = vec![];

    for account_key in account_keys {
        let account_address = ACCOUNTS().load(deps.storage, account_key)?.account;
        migration_msgs.push(WasmMsg::Migrate {
            contract_addr: account_address.to_string(),
            new_code_id: msg.warp_account_code_id.u64(),
            msg: to_binary(&account::MigrateMsg {})?,
        })
    }

    Ok(Response::new().add_messages(migration_msgs))
}

pub fn migrate_pending_jobs(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MigrateJobsMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let start_after = msg.start_after;
    let start_after = start_after.map(Bound::exclusive);

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
        .keys(deps.storage, start_after, None, Order::Ascending)
        .take(msg.limit as usize)
        .collect();
    let job_keys = job_keys?;

    for job_key in job_keys {
        let v1_job = V1_PENDING_JOBS().load(deps.storage, job_key)?;
        let mut new_vars = vec![];

        let job_vars: Vec<V1Variable> = serde_json_wasm::from_str(&v1_job.vars)
            .map_err(|e| StdError::generic_err(e.to_string()))?;

        for var in job_vars {
            new_vars.push(match var {
                V1Variable::Static(v) => Variable::Static(StaticVariable {
                    kind: v.kind,
                    name: v.name,
                    encode: v.encode,
                    init_fn: FnValue::String(StringValue::Simple(v.value.clone())),
                    reinitialize: false,
                    value: Some(v.value.clone()),
                    update_fn: v.update_fn,
                }),
                V1Variable::External(v) => Variable::External(ExternalVariable {
                    kind: v.kind,
                    name: v.name,
                    encode: v.encode,
                    init_fn: v.init_fn,
                    reinitialize: v.reinitialize,
                    value: v.value,
                    update_fn: v.update_fn,
                }),
                V1Variable::Query(v) => Variable::Query(QueryVariable {
                    kind: v.kind,
                    name: v.name,
                    encode: v.encode,
                    init_fn: v.init_fn,
                    reinitialize: v.reinitialize,
                    value: v.value,
                    update_fn: v.update_fn,
                }),
            })
        }

        let warp_account = ACCOUNTS().load(deps.storage, v1_job.owner.clone())?;

        PENDING_JOBS().save(
            deps.storage,
            job_key,
            &Job {
                id: v1_job.id,
                prev_id: None,
                owner: v1_job.owner,
                account: warp_account.account,
                last_update_time: v1_job.last_update_time,
                name: v1_job.name,
                description: v1_job.description,
                labels: v1_job.labels,
                status: v1_job.status,
                terminate_condition: None,
                executions: vec![Execution {
                    condition: v1_job.condition,
                    msgs: v1_job.msgs,
                }],
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

pub fn migrate_finished_jobs(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MigrateJobsMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let start_after = msg.start_after;
    let start_after = start_after.map(Bound::exclusive);

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
        .keys(deps.storage, start_after, None, Order::Ascending)
        .take(msg.limit as usize)
        .collect();
    let job_keys = job_keys?;

    for job_key in job_keys {
        let v1_job = V1_FINISHED_JOBS().load(deps.storage, job_key)?;
        let mut new_vars = vec![];

        let job_vars: Vec<V1Variable> = serde_json_wasm::from_str(&v1_job.vars)
            .map_err(|e| StdError::generic_err(e.to_string()))?;

        for var in job_vars {
            new_vars.push(match var {
                V1Variable::Static(v) => Variable::Static(StaticVariable {
                    kind: v.kind,
                    name: v.name,
                    encode: v.encode,
                    init_fn: FnValue::String(StringValue::Simple(v.value.clone())),
                    reinitialize: false,
                    value: Some(v.value.clone()),
                    update_fn: v.update_fn,
                }),
                V1Variable::External(v) => Variable::External(ExternalVariable {
                    kind: v.kind,
                    name: v.name,
                    encode: v.encode,
                    init_fn: v.init_fn,
                    reinitialize: v.reinitialize,
                    value: v.value,
                    update_fn: v.update_fn,
                }),
                V1Variable::Query(v) => Variable::Query(QueryVariable {
                    kind: v.kind,
                    name: v.name,
                    encode: v.encode,
                    init_fn: v.init_fn,
                    reinitialize: v.reinitialize,
                    value: v.value,
                    update_fn: v.update_fn,
                }),
            })
        }

        let warp_account = ACCOUNTS().load(deps.storage, v1_job.owner.clone())?;

        FINISHED_JOBS().save(
            deps.storage,
            job_key,
            &Job {
                id: v1_job.id,
                prev_id: None,
                owner: v1_job.owner,
                account: warp_account.account,
                last_update_time: v1_job.last_update_time,
                name: v1_job.name,
                description: v1_job.description,
                labels: v1_job.labels,
                status: v1_job.status,
                executions: vec![Execution {
                    condition: v1_job.condition,
                    msgs: v1_job.msgs,
                }],
                terminate_condition: None,
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

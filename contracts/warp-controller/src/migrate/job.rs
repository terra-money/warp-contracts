use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Order, Response, Uint128, Uint64};
use cw_storage_plus::{Bound, Index, IndexList, IndexedMap, MultiIndex, UniqueIndex};

use crate::{
    state::{FINISHED_JOBS, LEGACY_ACCOUNTS, PENDING_JOBS},
    ContractError,
};
use controller::{
    account::AssetInfo,
    job::{Job, JobStatus},
    Config, MigrateJobsMsg,
};

use resolver::{
    condition::{Condition, StringValue},
    variable::{
        ExternalExpr, ExternalVariable, FnValue, QueryExpr, QueryVariable, StaticVariable,
        UpdateFn, Variable, VariableKind,
    },
};

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

pub fn migrate_pending_jobs(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MigrateJobsMsg,
    config: Config,
) -> Result<Response, ContractError> {
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
        for var in v1_job.vars {
            new_vars.push(match var {
                V1Variable::Static(v) => Variable::Static(StaticVariable {
                    kind: v.kind,
                    name: v.name,
                    encode: false,
                    init_fn: FnValue::String(StringValue::Simple(v.value.clone())),
                    reinitialize: false,
                    value: Some(v.value.clone()),
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

        let warp_account = LEGACY_ACCOUNTS().load(deps.storage, v1_job.owner.clone())?;

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

    Ok(Response::new())
}

pub fn migrate_finished_jobs(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MigrateJobsMsg,
    config: Config,
) -> Result<Response, ContractError> {
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
        for var in v1_job.vars {
            new_vars.push(match var {
                V1Variable::Static(v) => Variable::Static(StaticVariable {
                    kind: v.kind,
                    name: v.name,
                    encode: false,
                    init_fn: FnValue::String(StringValue::Simple(v.value.clone())),
                    reinitialize: false,
                    value: Some(v.value.clone()),
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

        let warp_account = LEGACY_ACCOUNTS().load(deps.storage, v1_job.owner.clone())?;

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

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::state::{FINISHED_JOBS, LEGACY_ACCOUNTS, PENDING_JOBS};
use crate::{state::CONFIG, ContractError};

use controller::account::AssetInfo;
use controller::job::{Execution, Job, JobStatus};
use controller::MigrateJobsMsg;
use cosmwasm_std::{Addr, Order, Uint128, Uint64};
use cw_storage_plus::{Bound, Index, IndexList, IndexedMap, MultiIndex, UniqueIndex};

#[cw_serde]
pub struct OldJob {
    pub id: Uint64,
    pub owner: Addr,
    pub last_update_time: Uint64,
    pub name: String,
    pub description: String,
    pub labels: Vec<String>,
    pub status: JobStatus,
    pub terminate_condition: Option<String>,
    pub executions: Vec<Execution>,
    pub vars: String,
    pub recurring: bool,
    pub reward: Uint128,
    pub assets_to_withdraw: Vec<AssetInfo>,
}

pub struct OldJobIndexes<'a> {
    pub reward: UniqueIndex<'a, (u128, u64), OldJob>,
    pub publish_time: MultiIndex<'a, u64, OldJob, u64>,
}

impl IndexList<OldJob> for OldJobIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<OldJob>> + '_> {
        let v: Vec<&dyn Index<OldJob>> = vec![&self.reward, &self.publish_time];
        Box::new(v.into_iter())
    }
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
    pub fn OLD_PENDING_JOBS<'a>() -> IndexedMap<'a, u64, OldJob, OldJobIndexes<'a>> {
        let indexes = OldJobIndexes {
            reward: UniqueIndex::new(
                |job| (job.reward.u128(), job.id.u64()),
                "pending_jobs__reward_v4",
            ),
            publish_time: MultiIndex::new(
                |_pk, job| job.last_update_time.u64(),
                "pending_jobs_v4",
                "pending_jobs__publish_timestamp_v4",
            ),
        };
        IndexedMap::new("pending_jobs_v4", indexes)
    }

    let job_keys: Result<Vec<_>, _> = OLD_PENDING_JOBS()
        .keys(deps.storage, start_after, None, Order::Ascending)
        .take(msg.limit as usize)
        .collect();
    let job_keys = job_keys?;

    for job_key in job_keys {
        let old_job = OLD_PENDING_JOBS().load(deps.storage, job_key)?;
        let warp_account = LEGACY_ACCOUNTS().load(deps.storage, old_job.owner.clone())?;

        PENDING_JOBS().save(
            deps.storage,
            job_key,
            &Job {
                id: old_job.id,
                prev_id: None,
                owner: old_job.owner,
                account: warp_account.account,
                last_update_time: old_job.last_update_time,
                name: old_job.name,
                description: old_job.description,
                labels: old_job.labels,
                status: old_job.status,
                terminate_condition: None,
                executions: old_job.executions,
                vars: old_job.vars,
                recurring: old_job.recurring,
                reward: old_job.reward,
                assets_to_withdraw: old_job.assets_to_withdraw,
                duration_days: Uint128::from(30u128),
                created_at_time: old_job.last_update_time,
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
    pub fn OLD_FINISHED_JOBS<'a>() -> IndexedMap<'a, u64, OldJob, OldJobIndexes<'a>> {
        let indexes = OldJobIndexes {
            reward: UniqueIndex::new(
                |job| (job.reward.u128(), job.id.u64()),
                "finished_jobs__reward_v4",
            ),
            publish_time: MultiIndex::new(
                |_pk, job| job.last_update_time.u64(),
                "finished_jobs_v4",
                "finished_jobs__publish_timestamp_v4",
            ),
        };
        IndexedMap::new("finished_jobs_v4", indexes)
    }

    let job_keys: Result<Vec<_>, _> = OLD_FINISHED_JOBS()
        .keys(deps.storage, start_after, None, Order::Ascending)
        .take(msg.limit as usize)
        .collect();
    let job_keys = job_keys?;

    for job_key in job_keys {
        let old_job = OLD_FINISHED_JOBS().load(deps.storage, job_key)?;
        let warp_account = LEGACY_ACCOUNTS().load(deps.storage, old_job.owner.clone())?;

        FINISHED_JOBS().save(
            deps.storage,
            job_key,
            &Job {
                id: old_job.id,
                prev_id: None,
                owner: old_job.owner,
                account: warp_account.account,
                last_update_time: old_job.last_update_time,
                name: old_job.name,
                description: old_job.description,
                labels: old_job.labels,
                status: old_job.status,
                executions: old_job.executions,
                terminate_condition: None,
                vars: old_job.vars,
                recurring: old_job.recurring,
                reward: old_job.reward,
                assets_to_withdraw: old_job.assets_to_withdraw,
                duration_days: Uint128::from(30u128),
                created_at_time: old_job.last_update_time,
            },
        )?;
    }

    Ok(Response::new())
}

use cosmwasm_std::{Addr, DepsMut, Env, Uint64};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex, UniqueIndex};

use controller::{
    account::LegacyAccount,
    job::{Job, JobStatus, UpdateJobMsg},
    Config, State,
};

use crate::ContractError;

pub struct JobIndexes<'a> {
    pub reward: UniqueIndex<'a, (u128, u64), Job>,
    pub publish_time: MultiIndex<'a, u64, Job, u64>,
}

impl IndexList<Job> for JobIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Job>> + '_> {
        let v: Vec<&dyn Index<Job>> = vec![&self.reward, &self.publish_time];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn PENDING_JOBS<'a>() -> IndexedMap<'a, u64, Job, JobIndexes<'a>> {
    let indexes = JobIndexes {
        reward: UniqueIndex::new(
            |job| (job.reward.u128(), job.id.u64()),
            "pending_jobs__reward_v5",
        ),
        publish_time: MultiIndex::new(
            |_pk, job| job.last_update_time.u64(),
            "pending_jobs_v5",
            "pending_jobs__publish_timestamp_v5",
        ),
    };
    IndexedMap::new("pending_jobs_v5", indexes)
}

#[allow(non_snake_case)]
pub fn FINISHED_JOBS<'a>() -> IndexedMap<'a, u64, Job, JobIndexes<'a>> {
    let indexes = JobIndexes {
        reward: UniqueIndex::new(
            |job| (job.reward.u128(), job.id.u64()),
            "finished_jobs__reward_v5",
        ),
        publish_time: MultiIndex::new(
            |_pk, job| job.last_update_time.u64(),
            "finished_jobs_v5",
            "finished_jobs__publish_timestamp_v5",
        ),
    };
    IndexedMap::new("finished_jobs_v5", indexes)
}

pub struct LegacyAccountIndexes<'a> {
    pub account: UniqueIndex<'a, Addr, LegacyAccount>,
}

impl IndexList<LegacyAccount> for LegacyAccountIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<LegacyAccount>> + '_> {
        let v: Vec<&dyn Index<LegacyAccount>> = vec![&self.account];
        Box::new(v.into_iter())
    }
}

// !!! DEPRECATED !!!
// LEGACY_ACCOUNTS stores legacy account (all user's jobs share the same account, this is the old way of doing things before introducing job account)
// As of late 2023, we introduced job account meaning each job will have its own account, no more job sharing the same account
// We keep this for backward compatibility so user can withdraw from their old accounts
#[allow(non_snake_case)]
pub fn LEGACY_ACCOUNTS<'a>() -> IndexedMap<'a, Addr, LegacyAccount, LegacyAccountIndexes<'a>> {
    let indexes = LegacyAccountIndexes {
        account: UniqueIndex::new(|account| account.account.clone(), "accounts__account"),
    };
    IndexedMap::new("accounts", indexes)
}

pub const QUERY_PAGE_SIZE: u32 = 50;
pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");

pub struct JobQueue;

impl JobQueue {
    pub fn add(deps: &mut DepsMut, job: Job) -> Result<Job, ContractError> {
        let state = STATE.load(deps.storage)?;

        let job = PENDING_JOBS().update(deps.storage, state.current_job_id.u64(), |s| match s {
            None => Ok(job),
            Some(_) => Err(ContractError::JobAlreadyExists {}),
        })?;

        STATE.save(
            deps.storage,
            &State {
                current_job_id: state.current_job_id.checked_add(Uint64::new(1))?,
                q: state.q.checked_add(Uint64::new(1))?,
            },
        )?;

        Ok(job)
    }

    pub fn get(deps: &DepsMut, job_id: u64) -> Result<Job, ContractError> {
        let job = PENDING_JOBS().load(deps.storage, job_id)?;

        Ok(job)
    }

    pub fn sync(deps: &mut DepsMut, env: Env, job: Job) -> Result<Job, ContractError> {
        PENDING_JOBS().update(deps.storage, job.id.u64(), |j| match j {
            None => Err(ContractError::JobDoesNotExist {}),
            Some(job) => Ok(Job {
                id: job.id,
                prev_id: job.prev_id,
                owner: job.owner,
                account: job.account,
                last_update_time: Uint64::new(env.block.time.seconds()),
                name: job.name,
                description: job.description,
                labels: job.labels,
                status: JobStatus::Pending,
                executions: job.executions,
                terminate_condition: job.terminate_condition,
                vars: job.vars,
                recurring: job.recurring,
                requeue_on_evict: job.requeue_on_evict,
                reward: job.reward,
                assets_to_withdraw: job.assets_to_withdraw,
                duration_days: job.duration_days,
            }),
        })
    }

    pub fn update(deps: &mut DepsMut, _env: Env, data: UpdateJobMsg) -> Result<Job, ContractError> {
        PENDING_JOBS().update(deps.storage, data.id.u64(), |h| match h {
            None => Err(ContractError::JobDoesNotExist {}),
            Some(job) => Ok(Job {
                id: job.id,
                prev_id: job.prev_id,
                owner: job.owner,
                account: job.account,
                last_update_time: job.last_update_time,
                name: data.name.unwrap_or(job.name),
                description: data.description.unwrap_or(job.description),
                labels: data.labels.unwrap_or(job.labels),
                status: job.status,
                executions: job.executions,
                terminate_condition: job.terminate_condition,
                vars: job.vars,
                recurring: job.recurring,
                requeue_on_evict: job.requeue_on_evict,
                reward: job.reward,
                assets_to_withdraw: job.assets_to_withdraw,
                duration_days: job.duration_days,
            }),
        })
    }

    pub fn finalize(
        deps: &mut DepsMut,
        env: Env,
        job_id: u64,
        status: JobStatus,
    ) -> Result<Job, ContractError> {
        if status == JobStatus::Pending {
            return Err(ContractError::Unauthorized {});
        }

        let job = PENDING_JOBS().load(deps.storage, job_id)?;

        let new_job = Job {
            id: job.id,
            prev_id: job.prev_id,
            owner: job.owner,
            account: job.account,
            last_update_time: Uint64::new(env.block.time.seconds()),
            name: job.name,
            description: job.description,
            labels: job.labels,
            status,
            terminate_condition: job.terminate_condition,
            executions: job.executions,
            vars: job.vars,
            recurring: job.recurring,
            requeue_on_evict: job.requeue_on_evict,
            reward: job.reward,
            assets_to_withdraw: job.assets_to_withdraw,
            duration_days: job.duration_days,
        };

        FINISHED_JOBS().update(deps.storage, job_id, |j| match j {
            None => Ok(new_job.clone()),
            Some(_) => Err(ContractError::JobAlreadyFinished {}),
        })?;

        PENDING_JOBS().remove(deps.storage, job_id)?;

        let state = STATE.load(deps.storage)?;
        STATE.save(
            deps.storage,
            &State {
                current_job_id: state.current_job_id,
                q: state.q.checked_sub(Uint64::new(1))?,
            },
        )?;

        Ok(new_job)
    }
}

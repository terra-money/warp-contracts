use cosmwasm_std::{Env, Storage, Uint64};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex, UniqueIndex};

use controller::{
    job::{Job, JobStatus, UpdateJobMsg},
    Config, State,
};

use crate::ContractError;

pub struct JobIndexes<'a> {
    pub reward: UniqueIndex<'a, (u128, u64), Job>,
    pub publish_time: MultiIndex<'a, u64, Job, u64>,
    pub owner: MultiIndex<'a, String, Job, u64>,
}

impl IndexList<Job> for JobIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Job>> + '_> {
        let v: Vec<&dyn Index<Job>> = vec![&self.reward, &self.publish_time, &self.owner];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn PENDING_JOBS<'a>() -> IndexedMap<'a, u64, Job, JobIndexes<'a>> {
    let indexes = JobIndexes {
        reward: UniqueIndex::new(
            |job| (job.reward.u128(), job.id.u64()),
            "pending_jobs__reward_v6",
        ),
        publish_time: MultiIndex::new(
            |_pk, job| job.last_update_time.u64(),
            "pending_jobs_v6",
            "pending_jobs__publish_timestamp_v6",
        ),
        owner: MultiIndex::new(
            |_pk, job| job.owner.to_string(),
            "pending_jobs_v6",
            "pending_jobs__owner_v6",
        ),
    };
    IndexedMap::new("pending_jobs_v6", indexes)
}

#[allow(non_snake_case)]
pub fn FINISHED_JOBS<'a>() -> IndexedMap<'a, u64, Job, JobIndexes<'a>> {
    let indexes = JobIndexes {
        reward: UniqueIndex::new(
            |job| (job.reward.u128(), job.id.u64()),
            "finished_jobs__reward_v6",
        ),
        publish_time: MultiIndex::new(
            |_pk, job| job.last_update_time.u64(),
            "finished_jobs_v6",
            "finished_jobs__publish_timestamp_v6",
        ),
        owner: MultiIndex::new(
            |_pk, job| job.owner.to_string(),
            "finished_jobs_v6",
            "finished_jobs__owner_v6",
        ),
    };
    IndexedMap::new("finished_jobs_v6", indexes)
}

pub const QUERY_PAGE_SIZE: u32 = 50;
pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");

pub struct JobQueue;

impl JobQueue {
    pub fn add(storage: &mut dyn Storage, job: Job) -> Result<Job, ContractError> {
        let state: State = STATE.load(storage)?;

        let job = PENDING_JOBS().update(storage, state.current_job_id.u64(), |s| match s {
            None => Ok(job),
            Some(_) => Err(ContractError::JobAlreadyExists {}),
        })?;

        STATE.save(
            storage,
            &State {
                current_job_id: state.current_job_id.checked_add(Uint64::new(1))?,
                q: state.q.checked_add(Uint64::new(1))?,
            },
        )?;

        Ok(job)
    }

    pub fn get(storage: &dyn Storage, job_id: u64) -> Result<Job, ContractError> {
        let job = PENDING_JOBS().load(storage, job_id)?;

        Ok(job)
    }

    pub fn sync(storage: &mut dyn Storage, env: Env, job: Job) -> Result<Job, ContractError> {
        let res = PENDING_JOBS().update(storage, job.id.u64(), |j| match j {
            None => Err(ContractError::JobDoesNotExist {}),
            Some(_) => Ok(Job {
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
                reward: job.reward,
                assets_to_withdraw: job.assets_to_withdraw,
                duration_days: job.duration_days,
                created_at_time: Uint64::from(env.block.time.seconds()),
                funding_account: job.funding_account,
            }),
        })?;

        Ok(res)
    }

    pub fn update(
        storage: &mut dyn Storage,
        env: Env,
        data: UpdateJobMsg,
    ) -> Result<Job, ContractError> {
        let job = PENDING_JOBS().update(storage, data.id.u64(), |h| match h {
            None => Err(ContractError::JobDoesNotExist {}),
            Some(job) => Ok(Job {
                id: job.id,
                prev_id: job.prev_id,
                owner: job.owner,
                account: job.account,
                last_update_time: Uint64::new(env.block.time.seconds()),
                name: data.name.unwrap_or(job.name),
                description: data.description.unwrap_or(job.description),
                labels: data.labels.unwrap_or(job.labels),
                status: job.status,
                executions: job.executions,
                terminate_condition: job.terminate_condition,
                vars: job.vars,
                recurring: job.recurring,
                reward: job.reward,
                assets_to_withdraw: job.assets_to_withdraw,
                duration_days: job.duration_days,
                created_at_time: job.created_at_time,
                funding_account: job.funding_account,
            }),
        })?;

        Ok(job)
    }

    pub fn finalize(
        storage: &mut dyn Storage,
        env: Env,
        job_id: u64,
        status: JobStatus,
    ) -> Result<Job, ContractError> {
        if status == JobStatus::Pending {
            return Err(ContractError::Unauthorized {});
        }

        let job = PENDING_JOBS().load(storage, job_id)?;

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
            reward: job.reward,
            assets_to_withdraw: job.assets_to_withdraw,
            duration_days: job.duration_days,
            created_at_time: job.created_at_time,
            funding_account: job.funding_account,
        };

        FINISHED_JOBS().update(storage, job_id, |j| match j {
            None => Ok(new_job.clone()),
            Some(_) => Err(ContractError::JobAlreadyFinished {}),
        })?;

        PENDING_JOBS().remove(storage, job_id)?;

        let state = STATE.load(storage)?;
        STATE.save(
            storage,
            &State {
                current_job_id: state.current_job_id,
                q: state.q.checked_sub(Uint64::new(1))?,
            },
        )?;

        Ok(new_job)
    }
}

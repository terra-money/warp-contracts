use controller::account::Account;
use cosmwasm_std::{Addr, DepsMut, Env, Uint128, Uint64};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex, UniqueIndex};

use controller::job::{Job, JobStatus, UpdateJobMsg};
use controller::{Config, State};

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
            "pending_jobs__reward_v3",
        ),
        publish_time: MultiIndex::new(
            |_pk, job| job.last_update_time.u64(),
            "pending_jobs_v3",
            "pending_jobs__publish_timestamp_v3",
        ),
    };
    IndexedMap::new("pending_jobs_v3", indexes)
}

#[allow(non_snake_case)]
pub fn FINISHED_JOBS<'a>() -> IndexedMap<'a, u64, Job, JobIndexes<'a>> {
    let indexes = JobIndexes {
        reward: UniqueIndex::new(
            |job| (job.reward.u128(), job.id.u64()),
            "finished_jobs__reward_v3",
        ),
        publish_time: MultiIndex::new(
            |_pk, job| job.last_update_time.u64(),
            "finished_jobs_v3",
            "finished_jobs__publish_timestamp_v3",
        ),
    };
    IndexedMap::new("finished_jobs_v3", indexes)
}

pub struct AccountIndexes<'a> {
    pub account: UniqueIndex<'a, Addr, Account>,
}

impl IndexList<Account> for AccountIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Account>> + '_> {
        let v: Vec<&dyn Index<Account>> = vec![&self.account];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn ACCOUNTS<'a>() -> IndexedMap<'a, Addr, Account, AccountIndexes<'a>> {
    let indexes = AccountIndexes {
        account: UniqueIndex::new(|account| account.account.clone(), "accounts__account"),
    };
    IndexedMap::new("accounts", indexes)
}

pub const QUERY_PAGE_SIZE: u32 = 50;
pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");

pub trait JobQueue {
    fn add(deps: &mut DepsMut, job: Job) -> Result<Job, ContractError>;
    fn get(deps: &DepsMut, job_id: u64) -> Result<Job, ContractError>;
    fn sync(deps: &mut DepsMut, env: Env, job: Job) -> Result<Job, ContractError>;
    fn update(deps: &mut DepsMut, env: Env, data: UpdateJobMsg) -> Result<Job, ContractError>;
    fn finalize(
        deps: &mut DepsMut,
        env: Env,
        job_id: u64,
        status: JobStatus,
    ) -> Result<Job, ContractError>;
}

pub struct JobQueueInstance;

impl JobQueue for JobQueueInstance {
    fn add(deps: &mut DepsMut, job: Job) -> Result<Job, ContractError> {
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

    fn get(deps: &DepsMut, job_id: u64) -> Result<Job, ContractError> {
        let job = PENDING_JOBS().load(deps.storage, job_id)?;

        Ok(job)
    }

    fn sync(deps: &mut DepsMut, env: Env, job: Job) -> Result<Job, ContractError> {
        PENDING_JOBS().update(deps.storage, job.id.u64(), |j| match j {
            None => Err(ContractError::JobDoesNotExist {}),
            Some(job) => Ok(Job {
                id: job.id,
                owner: job.owner,
                last_update_time: Uint64::new(env.block.time.seconds()),
                name: job.name,
                description: job.description,
                labels: job.labels,
                status: JobStatus::Pending,
                condition: job.condition,
                terminate_condition: job.terminate_condition,
                msgs: job.msgs,
                vars: job.vars,
                recurring: job.recurring,
                requeue_on_evict: job.requeue_on_evict,
                reward: job.reward,
                assets_to_withdraw: job.assets_to_withdraw,
            }),
        })
    }

    fn update(deps: &mut DepsMut, env: Env, data: UpdateJobMsg) -> Result<Job, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        let added_reward: Uint128 = data.added_reward.unwrap_or(Uint128::new(0));

        PENDING_JOBS().update(deps.storage, data.id.u64(), |h| match h {
            None => Err(ContractError::JobDoesNotExist {}),
            Some(job) => Ok(Job {
                id: job.id,
                owner: job.owner,
                last_update_time: if added_reward > config.minimum_reward {
                    Uint64::new(env.block.time.seconds())
                } else {
                    job.last_update_time
                },
                name: data.name.unwrap_or(job.name),
                description: data.description.unwrap_or(job.description),
                labels: data.labels.unwrap_or(job.labels),
                status: job.status,
                condition: job.condition,
                terminate_condition: job.terminate_condition,
                msgs: job.msgs,
                vars: job.vars,
                recurring: job.recurring,
                requeue_on_evict: job.requeue_on_evict,
                reward: job.reward + added_reward,
                assets_to_withdraw: job.assets_to_withdraw,
            }),
        })
    }

    fn finalize(
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
            owner: job.owner,
            last_update_time: Uint64::new(env.block.time.seconds()),
            name: job.name,
            description: job.description,
            labels: job.labels,
            status,
            condition: job.condition,
            terminate_condition: job.terminate_condition,
            msgs: job.msgs,
            vars: job.vars,
            recurring: job.recurring,
            requeue_on_evict: job.requeue_on_evict,
            reward: job.reward,
            assets_to_withdraw: job.assets_to_withdraw,
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

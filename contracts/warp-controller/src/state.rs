use controller::account::Account;
use cosmwasm_std::Addr;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex, UniqueIndex};

use controller::job::Job;
use controller::{Config, State};

pub struct JobIndexes<'a> {
    pub reward: UniqueIndex<'a, (u128, u64), Job>,
    // publish_time is never read anywhere?
    // pub publish_time: MultiIndex<'a, u64, Job, u64>,
    pub owner: MultiIndex<'a, Addr, Job, String>,
}

impl IndexList<Job> for JobIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Job>> + '_> {
        // let v: Vec<&dyn Index<Job>> = vec![&self.reward, &self.publish_time, &self.owner];
        let v: Vec<&dyn Index<Job>> = vec![&self.reward, &self.owner];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn PENDING_JOBS<'a>() -> IndexedMap<'a, u64, Job, JobIndexes<'a>> {
    let indexes = JobIndexes {
        reward: UniqueIndex::new(
            |job| (job.reward.u128(), job.id.u64()),
            "pending_jobs__reward_v2",
        ),
        // publish_time: MultiIndex::new(
        //     |_pk, job| job.last_update_time.u64(),
        //     "pending_jobs_v2",
        //     "pending_jobs__publish_timestamp_v2",
        // ),
        owner: MultiIndex::new(
            |_pk, job| job.owner.clone(),
            "pending_jobs_v2",
            "pending_jobs__owner_v2",
        ),
    };
    IndexedMap::new("pending_jobs_v2", indexes)
}

#[allow(non_snake_case)]
pub fn FINISHED_JOBS<'a>() -> IndexedMap<'a, u64, Job, JobIndexes<'a>> {
    let indexes = JobIndexes {
        reward: UniqueIndex::new(
            |job| (job.reward.u128(), job.id.u64()),
            "finished_jobs__reward_v2",
        ),
        // publish_time: MultiIndex::new(
        //     |_pk, job| job.last_update_time.u64(),
        //     "finished_jobs_v2",
        //     "finished_jobs__publish_timestamp_v2",
        // ),
        owner: MultiIndex::new(
            |_pk, job| job.owner.clone(),
            "finished_jobs_v2",
            "finished_jobs__owner_v2",
        ),
    };
    IndexedMap::new("finished_jobs_v2", indexes)
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

// each address has 2 accounts

// FEE_ASSET_ACCOUNTS stores protocol fees (job reward, eviction fee) for job creator and reward for keeper
#[allow(non_snake_case)]
pub fn FEE_ASSET_ACCOUNTS<'a>() -> IndexedMap<'a, Addr, Account, AccountIndexes<'a>> {
    let indexes = AccountIndexes {
        account: UniqueIndex::new(|account| account.account.clone(), "fee_asset_accounts__account"),
    };
    IndexedMap::new("fee_asset_accounts", indexes)
}

// EXECUTION_ASSET_ACCOUNTS stores assets that will be used in the job execution
#[allow(non_snake_case)]
pub fn EXECUTION_ASSET_ACCOUNTS<'a>() -> IndexedMap<'a, Addr, Account, AccountIndexes<'a>> {
    let indexes = AccountIndexes {
        account: UniqueIndex::new(|account| account.account.clone(), "execution_asset_accounts__account"),
    };
    IndexedMap::new("execution_asset_accounts", indexes)
}

pub const QUERY_PAGE_SIZE: u32 = 50;
pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");

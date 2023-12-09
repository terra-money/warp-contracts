use account_tracker::{Config, FundingAccount};
use cosmwasm_std::{Addr, Uint64};
use cw_storage_plus::{Item, Map};

pub const CONFIG: Item<Config> = Item::new("config");

// Key is the (account owner address, account address), value is the ID of the pending job currently using it
pub const TAKEN_ACCOUNTS: Map<(&Addr, &Addr), Uint64> = Map::new("taken_accounts");

// Key is the (account owner address, account address), value is id of the last job which reserved it
pub const FREE_ACCOUNTS: Map<(&Addr, &Addr), Uint64> = Map::new("free_accounts");

// owner address -> funding_account[]
// - user can have multiple funding accounts
// - a job can be assigned to only one funding account
// - funding account can fund multiple jobs
pub const FUNDING_ACCOUNTS_BY_USER: Map<&Addr, Vec<FundingAccount>> =
    Map::new("funding_accounts_by_user");
pub const TAKEN_FUNDING_ACCOUNT_BY_JOB: Map<u64, Addr> = Map::new("funding_account_by_job");

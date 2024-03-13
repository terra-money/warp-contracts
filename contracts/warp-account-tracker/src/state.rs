use account_tracker::{Account, Config};
use cosmwasm_std::{Addr, Uint64};
use cw_storage_plus::{Item, Map};

pub const CONFIG: Item<Config> = Item::new("config");

// Key is the (account owner address, account address), value is the account struct
pub const ACCOUNTS: Map<(&Addr, &Addr), Account> = Map::new("accounts");

// Key is the (account owner address, account address), value is a vector of IDs of the jobs currently using it
pub const TAKEN_FUNDING_ACCOUNTS: Map<(&Addr, &Addr), Vec<Uint64>> =
    Map::new("taken_funding_accounts");

// Key is the (account owner address, account address), value is id of the last job which reserved it (vec[last_job_id])
pub const FREE_FUNDING_ACCOUNTS: Map<(&Addr, &Addr), Vec<Uint64>> =
    Map::new("free_funding_accounts");

// Key is the (account owner address, account address), value is the ID of the pending job currently using it
pub const TAKEN_JOB_ACCOUNTS: Map<(&Addr, &Addr), Uint64> = Map::new("taken_job_accounts");

// Key is the (account owner address, account address), value is id of the last job which reserved it
pub const FREE_JOB_ACCOUNTS: Map<(&Addr, &Addr), Uint64> = Map::new("free_job_accounts");

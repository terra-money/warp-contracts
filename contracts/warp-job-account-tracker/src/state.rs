use cosmwasm_std::{Addr, Uint64};
use cw_storage_plus::{Item, Map};
use job_account_tracker::Config;

pub const CONFIG: Item<Config> = Item::new("config");

// Key is the account address, value is the ID of the pending job currently using it
pub const TAKEN_ACCOUNTS: Map<&Addr, Uint64> = Map::new("taken_accounts");

// Key is the account address, value is a dummy data that is always true to make it behave like a set
pub const FREE_ACCOUNTS: Map<&Addr, bool> = Map::new("free_accounts");

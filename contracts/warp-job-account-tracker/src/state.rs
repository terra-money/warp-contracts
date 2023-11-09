use cosmwasm_std::{Addr, Uint64};
use cw_storage_plus::{Item, Map};
use job_account_tracker::Config;

pub const CONFIG: Item<Config> = Item::new("config");

// Key is the (account owner address, account address), value is the ID of the pending job currently using it
pub const TAKEN_ACCOUNTS: Map<(&Addr, &Addr), Uint64> = Map::new("taken_accounts");

// Key is the (account owner address, account address), value is id of the last job which reserved it
pub const FREE_ACCOUNTS: Map<(&Addr, &Addr), Uint64> = Map::new("free_accounts");

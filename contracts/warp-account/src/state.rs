use account::Config;
use cosmwasm_std::{Addr, Uint64};
use cw_storage_plus::{Item, Map};

pub const CONFIG: Item<Config> = Item::new("config");

// Key is the sub account address, value is the ID of the pending job currently using it
pub const OCCUPIED_SUB_ACCOUNTS: Map<&Addr, Uint64> = Map::new("in_use_sub_accounts");

// Key is the sub account address, value is a dummy data that is always true to make it behave like a set
pub const FREE_SUB_ACCOUNTS: Map<&Addr, bool> = Map::new("free_sub_accounts");

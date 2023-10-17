use warp_account_pkg::Config;
use cw_storage_plus::Item;

pub const CONFIG: Item<Config> = Item::new("config");

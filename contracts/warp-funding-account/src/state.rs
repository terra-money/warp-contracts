use cw_storage_plus::Item;
use funding_account::Config;

pub const CONFIG: Item<Config> = Item::new("config");

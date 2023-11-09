use cw_storage_plus::Item;
use legacy_account::Config;

pub const CONFIG: Item<Config> = Item::new("config");

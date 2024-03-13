use cw_storage_plus::Item;

use account::Config;

pub const CONFIG: Item<Config> = Item::new("config");

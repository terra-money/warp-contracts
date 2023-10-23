use cw_storage_plus::Item;

use job_account::Config;

pub const CONFIG: Item<Config> = Item::new("config");

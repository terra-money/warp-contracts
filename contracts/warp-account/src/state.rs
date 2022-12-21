use cw_storage_plus::Item;
use warp_protocol::account::Config;

pub const CONFIG: Item<Config> = Item::new("config");

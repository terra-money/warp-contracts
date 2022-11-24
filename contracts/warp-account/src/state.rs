use cw_storage_plus::Item;
use warp_protocol::account::account::Config;

pub const CONFIG: Item<Config> = Item::new("config");

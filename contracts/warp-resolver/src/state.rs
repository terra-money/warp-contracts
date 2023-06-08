use cw_storage_plus::{Item, Map};
use resolver::{Config, State, Template};

pub const CONFIG: Item<Config> = Item::new("config");
pub const TEMPLATES: Map<u64, Template> = Map::new("templates");
pub const STATE: Item<State> = Item::new("state");

pub const QUERY_PAGE_SIZE: u32 = 50;

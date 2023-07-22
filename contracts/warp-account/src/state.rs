use account::Config;
use controller::account::AssetInfoWithAmount;
use cw_storage_plus::{Item, Map};

pub const CONFIG: Item<Config> = Item::new("config");

// key is job id, value is total amount of assets locked for all pending jobs
pub const TOTAL_PENDING_JOBS_LOCKED_ASSETS: Item<AssetInfoWithAmount> =
    Item::new("total_pending_jobs_locked_assets");

// key is job id, value is total amount of assets locked for the pending job (and it's recurring jobs)
pub const PENDING_JOBS_LOCKED_ASSETS: Map<u64, AssetInfoWithAmount> =
    Map::new("pending_jobs_locked_assets");

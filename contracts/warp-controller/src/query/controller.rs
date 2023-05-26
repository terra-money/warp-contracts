use crate::state::CONFIG;

use cosmwasm_std::{Deps, Env, StdResult};

use warp_protocol::controller::{ConfigResponse, QueryConfigMsg};

pub fn query_config(deps: Deps, _env: Env, _data: QueryConfigMsg) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

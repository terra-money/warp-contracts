use crate::state::CONFIG;
use warp_controller_pkg::{ConfigResponse, QueryConfigMsg};
use cosmwasm_std::{Deps, Env, StdResult};

pub fn query_config(deps: Deps, _env: Env, _data: QueryConfigMsg) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

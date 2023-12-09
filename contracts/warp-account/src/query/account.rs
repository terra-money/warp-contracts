use crate::state::CONFIG;
use cosmwasm_std::{Deps, StdResult};
use account::ConfigResponse;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

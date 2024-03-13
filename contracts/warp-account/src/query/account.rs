use crate::state::CONFIG;
use account::ConfigResponse;
use cosmwasm_std::{Deps, StdResult};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

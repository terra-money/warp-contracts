use crate::state::{CONFIG, STATE};
use controller::{ConfigResponse, QueryConfigMsg, QueryStateMsg, StateResponse};
use cosmwasm_std::{Deps, Env, StdResult};

pub fn query_config(deps: Deps, _env: Env, _data: QueryConfigMsg) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

pub fn query_state(deps: Deps, _env: Env, _data: QueryStateMsg) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(StateResponse { state })
}

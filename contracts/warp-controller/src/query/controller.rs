use crate::state::CONFIG;
use crate::util::condition::resolve_query_expr;
use cosmwasm_std::{Deps, Env, StdError, StdResult};
use warp_protocol::controller::variable::QueryExpr;
use warp_protocol::controller::{
    ConfigResponse, QueryConfigMsg, SimulateQueryMsg, SimulateResponse,
};

pub fn query_config(deps: Deps, _env: Env, _data: QueryConfigMsg) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

pub fn query_simulate_query(
    deps: Deps,
    env: Env,
    data: SimulateQueryMsg,
) -> StdResult<SimulateResponse> {
    Ok(SimulateResponse {
        response: resolve_query_expr(
            deps,
            env,
            QueryExpr {
                selector: "".to_string(),
                query: data.query,
            },
        )
        .map_err(|e| StdError::generic_err(e.to_string()))?,
    })
}

use crate::state::CONFIG;
use crate::util::condition::resolve_query_expr;
use cosmwasm_std::{Deps, Env, StdError, StdResult};
use warp_protocol::controller::variable::QueryExpr;
use warp_protocol::controller::{
    ConfigResponse, QueryConfigMsg,
};

pub fn query_config(deps: Deps, _env: Env, _data: QueryConfigMsg) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

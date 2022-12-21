use crate::state::PENDING_JOBS;
use crate::util::condition::resolve_cond;
use cosmwasm_std::{Deps, Env, StdError, StdResult};
use warp_protocol::controller::condition::QueryResolveConditionMsg;
use warp_protocol::controller::job::QueryResolveJobConditionMsg;

pub fn query_resolve_condition(
    deps: Deps,
    env: Env,
    data: QueryResolveConditionMsg,
) -> StdResult<bool> {
        resolve_cond(deps, env, data.condition)
            .map_err(|e| StdError::generic_err(e.to_string()))
}

pub fn query_condition_active(
    deps: Deps,
    env: Env,
    data: QueryResolveJobConditionMsg,
) -> StdResult<bool> {
    let job = PENDING_JOBS().load(deps.storage, data.id.u64())?;
    let resp =
        resolve_cond(deps, env, job.condition).map_err(|e| StdError::generic_err(e.to_string()))?;
    Ok(resp)
}

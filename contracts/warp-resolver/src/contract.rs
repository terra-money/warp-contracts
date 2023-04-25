use crate::state::CONFIG;
use crate::ContractError;
use cosmwasm_std::{entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, StdError};
use warp_protocol::resolver::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::condition::resolve_cond;
use crate::variable::{apply_var_fn, hydrate_msgs, hydrate_vars, validate_vars_and_msgs};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // CONFIG.save(
    //     deps.storage,
    //     &Config {
    //         owner: deps.api.addr_validate(&msg.owner)?,
    //         warp_addr: info.sender,
    //     },
    // )?;
    // Ok(Response::new()
    //     .add_attribute("action", "instantiate")
    //     .add_attribute("contract_addr", env.contract.address)
    //     .add_attribute("owner", msg.owner)
    //     .add_attribute("funds", serde_json_wasm::to_string(&info.funds)?)
    //     .add_attribute("cw_funds", serde_json_wasm::to_string(&msg.funds)?)
    // )
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    Err(ContractError::Unauthorized {})
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::ApplyVarFns(data) => Ok(to_binary(&apply_var_fn(deps, env, data.vars, data.status)?)?),
        QueryMsg::ResolveCondition(data) => Ok(to_binary(&resolve_cond(deps, env, data.condition, &data.vars)?)?),
        // // QueryMsg::VarsValid(data) => {}
        // // QueryMsg::HasDuplicates(data) => {}
        // // QueryMsg::StringVarsInVector(data) => {}
        // // QueryMsg::AllVectorVarsPresent(data) => {}
        // // QueryMsg::MsgsValid(data) => {}
        QueryMsg::ValidateVarsAndMsgs(data) => Ok(to_binary(&validate_vars_and_msgs(data.vars, data.cond_string, data.msg_string)?)?),
        QueryMsg::HydrateVars(data) => Ok(to_binary(&hydrate_vars(deps, env, data.vars, data.external_inputs)?)?),
        QueryMsg::HydrateMsgs(data) => Ok(to_binary(&hydrate_msgs(data.msgs, data.vars)?)?)
    }
}

pub fn migrate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.warp_addr {
        return Err(ContractError::Unauthorized {});
    }
    Ok(Response::new())
}

use crate::util::condition::{resolve_cond, resolve_query_expr};
use crate::util::variable::{
    apply_var_fn, has_duplicates, hydrate_msgs, hydrate_vars, msgs_valid, string_vars_in_vector,
    vars_valid,
};
use crate::ContractError;
use cosmwasm_std::{
    entry_point, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,
};

use resolver::condition::Condition;
use resolver::variable::{QueryExpr, Variable};
use resolver::{
    ExecuteApplyVarFnMsg, ExecuteHydrateMsgsMsg, ExecuteHydrateVarsMsg, ExecuteMsg,
    ExecuteResolveConditionMsg, ExecuteSimulateQueryMsg, ExecuteValidateJobCreationMsg,
    InstantiateMsg, MigrateMsg, QueryApplyVarFnMsg, QueryHydrateMsgsMsg, QueryHydrateVarsMsg,
    QueryMsg, QueryResolveConditionMsg, QueryValidateJobCreationMsg, SimulateQueryMsg,
    SimulateResponse,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ExecuteSimulateQuery(msg) => execute_simulate_query(deps, env, info, msg),
        ExecuteMsg::ExecuteValidateJobCreation(data) => {
            execute_validate_job_creation(deps, env, info, data)
        }
        ExecuteMsg::ExecuteHydrateVars(data) => execute_hydrate_vars(deps, env, info, data),
        ExecuteMsg::ExecuteResolveCondition(data) => {
            execute_resolve_condition(deps, env, info, data)
        }
        ExecuteMsg::ExecuteApplyVarFn(data) => execute_apply_var_fn(deps, env, info, data),
        ExecuteMsg::ExecuteHydrateMsgs(data) => execute_hydrate_msgs(deps, env, info, data),
    }
}

pub fn execute_simulate_query(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteSimulateQueryMsg,
) -> Result<Response, ContractError> {
    let result = query_simulate_query(deps.as_ref(), env, SimulateQueryMsg { query: msg.query })?;

    Ok(Response::new()
        .add_attribute("action", "execute_simulate_query")
        .add_attribute("response", result.response))
}

pub fn execute_validate_job_creation(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    data: ExecuteValidateJobCreationMsg,
) -> Result<Response, ContractError> {
    let result = query_validate_job_creation(
        deps.as_ref(),
        env,
        QueryValidateJobCreationMsg {
            condition: data.condition,
            terminate_condition: data.terminate_condition,
            vars: data.vars,
            msgs: data.msgs,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "execute_validate_job_creation")
        .add_attribute(
            "response",
            if result.is_empty() {
                "valid"
            } else {
                "invalid"
            },
        ))
}

pub fn execute_hydrate_vars(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    data: ExecuteHydrateVarsMsg,
) -> Result<Response, ContractError> {
    let result = query_hydrate_vars(
        deps.as_ref(),
        env,
        QueryHydrateVarsMsg {
            vars: data.vars,
            external_inputs: data.external_inputs,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "execute_hydrate_vars")
        .add_attribute("response", &result))
}

pub fn execute_resolve_condition(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    data: ExecuteResolveConditionMsg,
) -> Result<Response, ContractError> {
    let result = query_resolve_condition(
        deps.as_ref(),
        env,
        QueryResolveConditionMsg {
            condition: data.condition,
            vars: data.vars,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "execute_resolve_condition")
        .add_attribute("response", result.to_string()))
}

pub fn execute_apply_var_fn(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    data: ExecuteApplyVarFnMsg,
) -> Result<Response, ContractError> {
    let result = query_apply_var_fn(
        deps.as_ref(),
        env,
        QueryApplyVarFnMsg {
            vars: data.vars,
            status: data.status,
        },
    )?;
    Ok(Response::new()
        .add_attribute("action", "execute_apply_var_fn")
        .add_attribute("response", &result))
}

pub fn execute_hydrate_msgs(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    data: ExecuteHydrateMsgsMsg,
) -> Result<Response, ContractError> {
    let result = query_hydrate_msgs(
        deps.as_ref(),
        env,
        QueryHydrateMsgsMsg {
            msgs: data.msgs,
            vars: data.vars,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "execute_hydrate_msgs")
        .add_attribute("response", serde_json_wasm::to_string(&result)?))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::SimulateQuery(data) => to_binary(&query_simulate_query(deps, env, data)?),
        QueryMsg::QueryValidateJobCreation(data) => {
            to_binary(&query_validate_job_creation(deps, env, data)?)
        }
        QueryMsg::QueryHydrateVars(data) => to_binary(&query_hydrate_vars(deps, env, data)?),
        QueryMsg::QueryResolveCondition(data) => {
            to_binary(&query_resolve_condition(deps, env, data)?)
        }
        QueryMsg::QueryApplyVarFn(data) => to_binary(&query_apply_var_fn(deps, env, data)?),
        QueryMsg::QueryHydrateMsgs(data) => to_binary(&query_hydrate_msgs(deps, env, data)?),
    }
}

fn query_validate_job_creation(
    _deps: Deps,
    _env: Env,
    data: QueryValidateJobCreationMsg,
) -> StdResult<String> {
    let _condition: Condition = serde_json_wasm::from_str(&data.condition)
        .map_err(|e| StdError::generic_err(format!("Condition input invalid: {}", e)))?;
    let terminate_condition_str = data.terminate_condition.clone().unwrap_or("".to_string());
    if !terminate_condition_str.is_empty() {
        let _terminate_condition: Condition = serde_json_wasm::from_str(&terminate_condition_str)
            .map_err(|e| {
            StdError::generic_err(format!("Terminate condition input invalid: {}", e))
        })?;
    }
    let vars: Vec<Variable> = serde_json_wasm::from_str(&data.vars)
        .map_err(|e| StdError::generic_err(format!("Vars input invalid: {}", e)))?;

    if !vars_valid(&vars) {
        return Err(StdError::generic_err(
            ContractError::InvalidVariables {}.to_string(),
        ));
    }

    if has_duplicates(&vars) {
        return Err(StdError::generic_err(
            ContractError::VariablesContainDuplicates {}.to_string(),
        ));
    }

    if !(string_vars_in_vector(&vars, &data.condition)
        && string_vars_in_vector(&vars, &terminate_condition_str)
        && string_vars_in_vector(&vars, &data.msgs))
    {
        return Err(StdError::generic_err(
            ContractError::VariablesMissingFromVector {}.to_string(),
        ));
    }

    if !msgs_valid(&data.msgs, &vars).map_err(|e| StdError::generic_err(e.to_string()))? {
        return Err(StdError::generic_err(
            ContractError::MsgError {
                msg: "msgs are invalid".to_string(),
            }
            .to_string(),
        ));
    }

    Ok("".to_string())
}

fn query_hydrate_vars(deps: Deps, env: Env, data: QueryHydrateVarsMsg) -> StdResult<String> {
    let vars: Vec<Variable> =
        serde_json_wasm::from_str(&data.vars).map_err(|e| StdError::generic_err(e.to_string()))?;
    serde_json_wasm::to_string(
        &hydrate_vars(deps, env, vars, data.external_inputs)
            .map_err(|e| StdError::generic_err(e.to_string()))?,
    )
    .map_err(|e| StdError::generic_err(e.to_string()))
}

fn query_resolve_condition(
    deps: Deps,
    env: Env,
    data: QueryResolveConditionMsg,
) -> StdResult<bool> {
    let condition: Condition = serde_json_wasm::from_str(&data.condition)
        .map_err(|e| StdError::generic_err(e.to_string()))?;
    let vars: Vec<Variable> =
        serde_json_wasm::from_str(&data.vars).map_err(|e| StdError::generic_err(e.to_string()))?;

    resolve_cond(deps, env, condition, &vars).map_err(|e| StdError::generic_err(e.to_string()))
}

fn query_apply_var_fn(deps: Deps, env: Env, data: QueryApplyVarFnMsg) -> StdResult<String> {
    let vars: Vec<Variable> =
        serde_json_wasm::from_str(&data.vars).map_err(|e| StdError::generic_err(e.to_string()))?;

    apply_var_fn(deps, env, vars, data.status).map_err(|e| StdError::generic_err(e.to_string()))
}

fn query_hydrate_msgs(
    _deps: Deps,
    _env: Env,
    data: QueryHydrateMsgsMsg,
) -> StdResult<Vec<CosmosMsg>> {
    let vars: Vec<Variable> =
        serde_json_wasm::from_str(&data.vars).map_err(|e| StdError::generic_err(e.to_string()))?;

    hydrate_msgs(data.msgs, vars).map_err(|e| StdError::generic_err(e.to_string()))
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
}

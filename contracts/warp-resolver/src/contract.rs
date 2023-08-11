use crate::util::condition::{resolve_cond, resolve_query_expr};
use crate::util::variable::{
    all_vector_vars_present, apply_var_fn, has_duplicates, hydrate_msgs, hydrate_vars, msgs_valid,
    string_vars_in_vector, vars_valid,
};
use crate::ContractError;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

use resolver::condition::Condition;
use resolver::variable::{QueryExpr, Variable};
use resolver::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SimulateQueryMsg, SimulateResponse,
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
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    Err(ContractError::Unauthorized {})
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::SimulateQuery(data) => to_binary(&query_simulate_query(deps, env, data)?),

        QueryMsg::QueryValidateJobCreation(data) => {
            let _condition: Condition = serde_json_wasm::from_str(&data.condition)
                .map_err(|e| StdError::generic_err(format!("Condition input invalid: {}", e)))?;
            let terminate_condition_str =
                data.terminate_condition.clone().unwrap_or("".to_string());
            if !terminate_condition_str.is_empty() {
                let _terminate_condition: Condition =
                    serde_json_wasm::from_str(&terminate_condition_str).map_err(|e| {
                        StdError::generic_err(format!("Terminate condition input invalid: {}", e))
                    })?;
            }
            let vars: Vec<Variable> = serde_json_wasm::from_str(&data.vars)
                .map_err(|e| StdError::generic_err(format!("Vars input invalid: {}", e)))?;
            let msgs: Vec<String> = serde_json_wasm::from_str(&data.msgs)
                .map_err(|e| StdError::generic_err(format!("Msgs input invalid: {}", e)))?;

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

            if !(string_vars_in_vector(&vars, &data.condition) //stringified
            && string_vars_in_vector(&vars, &terminate_condition_str) //stringified
            && string_vars_in_vector(&vars, &data.msgs))
            {
                return Err(StdError::generic_err(
                    ContractError::VariablesMissingFromVector {}.to_string(),
                ));
            }

            if !all_vector_vars_present(
                &vars,
                format!("{}{}{}", data.condition, terminate_condition_str, data.msgs),
            ) {
                return Err(StdError::generic_err(
                    ContractError::ExcessVariablesInVector {}.to_string(),
                ));
            }

            if !msgs_valid(&msgs, &vars).map_err(|e| StdError::generic_err(e.to_string()))? {
                return Err(StdError::generic_err(
                    ContractError::MsgError {
                        msg: "msgs are invalid".to_string(),
                    }
                    .to_string(),
                ));
            }
            to_binary(&"")
        }
        QueryMsg::QueryHydrateVars(data) => {
            let vars: Vec<Variable> = serde_json_wasm::from_str(&data.vars)
                .map_err(|e| StdError::generic_err(e.to_string()))?;
            to_binary(
                &serde_json_wasm::to_string(
                    &hydrate_vars(deps, env, vars, &data.external_inputs)
                        .map_err(|e| StdError::generic_err(e.to_string()))?,
                )
                .map_err(|e| StdError::generic_err(e.to_string()))?,
            )
        }
        QueryMsg::QueryResolveCondition(data) => {
            let condition: Condition = serde_json_wasm::from_str(&data.condition)
                .map_err(|e| StdError::generic_err(e.to_string()))?;
            let vars: Vec<Variable> = serde_json_wasm::from_str(&data.vars)
                .map_err(|e| StdError::generic_err(e.to_string()))?;
            to_binary(
                &resolve_cond(deps, env, condition, &vars)
                    .map_err(|e| StdError::generic_err(e.to_string()))?,
            )
        }
        QueryMsg::QueryApplyVarFn(data) => {
            let vars: Vec<Variable> = serde_json_wasm::from_str(&data.vars)
                .map_err(|e| StdError::generic_err(e.to_string()))?;
            to_binary(
                &apply_var_fn(deps, env, vars, data.status)
                    .map_err(|e| StdError::generic_err(e.to_string()))?,
            )
        }
        QueryMsg::QueryHydrateMsgs(data) => {
            let vars: Vec<Variable> = serde_json_wasm::from_str(&data.vars)
                .map_err(|e| StdError::generic_err(e.to_string()))?;
            let msgs: Vec<String> = serde_json_wasm::from_str(&data.msgs)
                .map_err(|e| StdError::generic_err(e.to_string()))?;
            to_binary(&hydrate_msgs(msgs, vars).map_err(|e| StdError::generic_err(e.to_string()))?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
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

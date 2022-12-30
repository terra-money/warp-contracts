use crate::util::condition::{
    resolve_query_expr_bool, resolve_query_expr_decimal,
    resolve_query_expr_int, resolve_query_expr_string, resolve_query_expr_uint,
};
use crate::ContractError;
use cosmwasm_std::{CosmosMsg, Deps, Env};

use warp_protocol::controller::job::ExternalInput;
use warp_protocol::controller::variable::{Variable, VariableKind};

pub fn hydrate_vars(
    deps: Deps,
    env: Env,
    vars: Vec<Variable>,
    external_inputs: Option<Vec<ExternalInput>>,
) -> Result<Vec<Variable>, ContractError> {
    let mut hydrated_vars = vec![];

    for var in vars {
        let hydrated_var = match var {
            Variable::Static(v) => {
                if v.value.is_none() && v.default_value.is_none() {
                    return Err(ContractError::Unauthorized {});
                }
                Variable::Static(v)
            }
            Variable::External(mut v) => {
                match external_inputs {
                    None => {
                        if v.value.is_none() && v.default_value.is_none() {
                            return Err(ContractError::Unauthorized {});
                        }
                        Variable::External(v)
                    }
                    Some(ref input) => {
                        let idx = input.iter().position(|i| i.name == v.name);
                        v.value = match idx {
                            None => return Err(ContractError::Unauthorized {}), //todo: err
                            Some(i) => Some(input[i].input.clone()),
                        };
                        if v.value.is_none() && v.default_value.is_none() {
                            return Err(ContractError::Unauthorized {});
                        }
                        Variable::External(v)
                    }
                }
            }
            Variable::Query(mut v) => {
                match v.kind {
                    VariableKind::String => {
                        v.value = Some(format!(
                            "\"{}\"",
                            resolve_query_expr_string(deps, env.clone(), v.call_fn.clone())?
                        ))
                    }
                    VariableKind::Uint => {
                        v.value = Some(format!(
                            "\"{}\"",
                            resolve_query_expr_uint(deps, env.clone(), v.call_fn.clone())?
                        ))
                    }
                    VariableKind::Int => {
                        v.value = Some(format!(
                            "{}",
                            resolve_query_expr_int(deps, env.clone(), v.call_fn.clone())?
                        ))
                    }
                    VariableKind::Decimal => {
                        v.value = Some(format!(
                            "\"{}\"",
                            resolve_query_expr_decimal(deps, env.clone(), v.call_fn.clone())?
                        ))
                    }
                    VariableKind::Timestamp => {
                        v.value = Some(format!(
                            "{}",
                            resolve_query_expr_int(deps, env.clone(), v.call_fn.clone())?
                        )) //todo: make sure this is right
                    }
                    VariableKind::Bool => {
                        v.value = Some(format!(
                            "{}",
                            resolve_query_expr_bool(deps, env.clone(), v.call_fn.clone())?
                        ))
                    }
                    VariableKind::Amount => {
                        v.value = Some(format!(
                            "\"{}\"",
                            resolve_query_expr_uint(deps, env.clone(), v.call_fn.clone())?
                        ))
                    }
                    VariableKind::Asset => {
                        v.value = Some(format!(
                            "\"{}\"",
                            resolve_query_expr_string(deps, env.clone(), v.call_fn.clone())?
                        ))
                    }
                }
                if v.value.is_none() && v.default_value.is_none() {
                    return Err(ContractError::Unauthorized {});
                }
                Variable::Query(v)
            }
        };
        hydrated_vars.push(hydrated_var);
    }
    Ok(hydrated_vars)
}

pub fn hydrate_msgs(
    msgs: Vec<String>,
    vars: Vec<Variable>,
) -> Result<Vec<CosmosMsg>, ContractError> {
    //todo:
    let mut parsed_msgs: Vec<CosmosMsg> = vec![];
    for mut msg in msgs {
        for var in &vars {
            let (name, replacement) = match var {
                Variable::Static(v) => {
                    match v.value.clone() {
                        None => {
                            match v.default_value.clone() {
                                None => return Err(ContractError::Unauthorized {}), //todo: err
                                Some(val) => (v.name.clone(), val),
                            }
                        }
                        Some(val) => (v.name.clone(), val),
                    }
                }
                Variable::External(v) => {
                    match v.value.clone() {
                        None => {
                            match v.default_value.clone() {
                                None => return Err(ContractError::Unauthorized {}), //todo: err
                                Some(val) => (v.name.clone(), val),
                            }
                        }
                        Some(val) => (v.name.clone(), val),
                    }
                }
                Variable::Query(v) => {
                    match v.value.clone() {
                        None => {
                            match v.default_value.clone() {
                                None => return Err(ContractError::Unauthorized {}), //todo: err
                                Some(val) => (v.name.clone(), val),
                            }
                        }
                        Some(val) => (v.name.clone(), val),
                    }
                }
            };
            msg = msg.replace(&format!("\"$WARPVAR.{}\"", name), &replacement);
        }
        parsed_msgs.push(serde_json_wasm::from_str::<CosmosMsg>(&msg)?)
    }

    Ok(parsed_msgs)
}

pub fn get_var(name: String, vars: &Vec<Variable>) -> Result<&Variable, ContractError> {
    for var in vars {
        let n = match var {
            Variable::Static(v) => v.name.clone(),
            Variable::External(v) => v.name.clone(),
            Variable::Query(v) => v.name.clone(),
        };
        if n == name {
            return Ok(var);
        }
    }
    Err(ContractError::Unauthorized {}) //todo: err
}

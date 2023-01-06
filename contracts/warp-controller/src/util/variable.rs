use crate::util::condition::{
    resolve_num_value_decimal, resolve_num_value_int, resolve_num_value_uint,
    resolve_query_expr_bool, resolve_query_expr_decimal, resolve_query_expr_int,
    resolve_query_expr_string, resolve_query_expr_uint, resolve_ref_bool,
};
use crate::ContractError;
use cosmwasm_std::{CosmosMsg, Deps, Env};

use warp_protocol::controller::job::{ExternalInput, JobStatus};
use warp_protocol::controller::variable::{UpdateFnValue, Variable, VariableKind};

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
                if v.value.is_none() {
                    return Err(ContractError::Unauthorized {});
                }
                Variable::Static(v)
            }
            Variable::External(mut v) => {
                match external_inputs {
                    None => {
                        if v.value.is_none() {
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
                        if v.value.is_none() {
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
                            resolve_query_expr_string(deps, env.clone(), v.init_fn.clone())?
                        ))
                    }
                    VariableKind::Uint => {
                        v.value = Some(format!(
                            "\"{}\"",
                            resolve_query_expr_uint(deps, env.clone(), v.init_fn.clone())?
                        ))
                    }
                    VariableKind::Int => {
                        v.value = Some(format!(
                            "{}",
                            resolve_query_expr_int(deps, env.clone(), v.init_fn.clone())?
                        ))
                    }
                    VariableKind::Decimal => {
                        v.value = Some(format!(
                            "\"{}\"",
                            resolve_query_expr_decimal(deps, env.clone(), v.init_fn.clone())?
                        ))
                    }
                    VariableKind::Timestamp => {
                        v.value = Some(format!(
                            "{}",
                            resolve_query_expr_int(deps, env.clone(), v.init_fn.clone())?
                        ))
                    }
                    VariableKind::Bool => {
                        v.value = Some(format!(
                            "{}",
                            resolve_query_expr_bool(deps, env.clone(), v.init_fn.clone())?
                        ))
                    }
                    VariableKind::Amount => {
                        v.value = Some(format!(
                            "\"{}\"",
                            resolve_query_expr_uint(deps, env.clone(), v.init_fn.clone())?
                        ))
                    }
                    VariableKind::Asset => {
                        v.value = Some(format!(
                            "\"{}\"",
                            resolve_query_expr_string(deps, env.clone(), v.init_fn.clone())?
                        ))
                    }
                }
                if v.value.is_none() {
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
    let mut parsed_msgs: Vec<CosmosMsg> = vec![];
    for mut msg in msgs {
        for var in &vars {
            let (name, replacement) = match var {
                Variable::Static(v) => {
                    match v.value.clone() {
                        None => {
                            return Err(ContractError::Unauthorized {}); //todo: err
                        }
                        Some(val) => (v.name.clone(), val),
                    }
                }
                Variable::External(v) => {
                    match v.value.clone() {
                        None => {
                            return Err(ContractError::Unauthorized {}); //todo: err
                        }
                        Some(val) => (v.name.clone(), val),
                    }
                }
                Variable::Query(v) => {
                    match v.value.clone() {
                        None => {
                            return Err(ContractError::Unauthorized {}); //todo: err
                        }
                        Some(val) => (v.name.clone(), val),
                    }
                }
            };
            msg = msg.replace(&format!("\"$warp.variable.{}\"", name), &replacement);
        }
        parsed_msgs.push(serde_json_wasm::from_str::<CosmosMsg>(&msg)?)
    }

    Ok(parsed_msgs)
}

pub fn apply_var_fn(
    deps: Deps,
    env: Env,
    vars: Vec<Variable>,
    status: JobStatus,
) -> Result<Vec<Variable>, ContractError> {
    for var in vars.clone() {
        match var {
            Variable::Static(mut v) => {
                match v.update_fn.clone() {
                    None => (),
                    Some(update_fn) => {
                        match status {
                            JobStatus::Pending => return Err(ContractError::Unauthorized {}), //todo: err
                            JobStatus::Executed => {
                                match update_fn.on_success {
                                    None => (),
                                    Some(on_success) => {
                                        match on_success {
                                            UpdateFnValue::Uint(nv) => {
                                                if v.kind != VariableKind::Uint {
                                                    return Err(ContractError::Unauthorized {});
                                                    //todo: err
                                                }
                                                v.value = Some(
                                                    resolve_num_value_uint(
                                                        deps,
                                                        env.clone(),
                                                        nv,
                                                        &vars,
                                                    )?
                                                    .to_string(),
                                                )
                                            }
                                            UpdateFnValue::Int(nv) => {
                                                if v.kind != VariableKind::Int {
                                                    return Err(ContractError::Unauthorized {});
                                                    //todo: err
                                                }
                                                v.value = Some(
                                                    resolve_num_value_int(
                                                        deps,
                                                        env.clone(),
                                                        nv,
                                                        &vars,
                                                    )?
                                                    .to_string(),
                                                )
                                            }
                                            UpdateFnValue::Decimal(nv) => {
                                                if v.kind != VariableKind::Uint {
                                                    return Err(ContractError::Unauthorized {});
                                                    //todo: err
                                                }
                                                v.value = Some(
                                                    resolve_num_value_decimal(
                                                        deps,
                                                        env.clone(),
                                                        nv,
                                                        &vars,
                                                    )?
                                                    .to_string(),
                                                )
                                            }
                                            UpdateFnValue::Timestamp(nv) => {
                                                if v.kind != VariableKind::Int {
                                                    return Err(ContractError::Unauthorized {});
                                                    //todo: err
                                                }
                                                v.value = Some(
                                                    resolve_num_value_int(
                                                        deps,
                                                        env.clone(),
                                                        nv,
                                                        &vars,
                                                    )?
                                                    .to_string(),
                                                )
                                            }
                                            UpdateFnValue::BlockHeight(nv) => {
                                                if v.kind != VariableKind::Int {
                                                    return Err(ContractError::Unauthorized {});
                                                    //todo: err
                                                }
                                                v.value = Some(
                                                    resolve_num_value_int(
                                                        deps,
                                                        env.clone(),
                                                        nv,
                                                        &vars,
                                                    )?
                                                    .to_string(),
                                                )
                                            }
                                            UpdateFnValue::Bool(val) => {
                                                if v.kind != VariableKind::Bool {
                                                    return Err(ContractError::Unauthorized {});
                                                    //todo: err
                                                }
                                                v.value = Some(
                                                    resolve_ref_bool(
                                                        deps,
                                                        env.clone(),
                                                        val,
                                                        &vars,
                                                    )?
                                                    .to_string(),
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                            JobStatus::Failed => {
                                match update_fn.on_error {
                                    None => (),
                                    Some(on_success) => {
                                        match on_success {
                                            UpdateFnValue::Uint(nv) => {
                                                if v.kind != VariableKind::Uint {
                                                    return Err(ContractError::Unauthorized {});
                                                    //todo: err
                                                }
                                                v.value = Some(
                                                    resolve_num_value_uint(
                                                        deps,
                                                        env.clone(),
                                                        nv,
                                                        &vars,
                                                    )?
                                                    .to_string(),
                                                )
                                            }
                                            UpdateFnValue::Int(nv) => {
                                                if v.kind != VariableKind::Int {
                                                    return Err(ContractError::Unauthorized {});
                                                    //todo: err
                                                }
                                                v.value = Some(
                                                    resolve_num_value_int(
                                                        deps,
                                                        env.clone(),
                                                        nv,
                                                        &vars,
                                                    )?
                                                    .to_string(),
                                                )
                                            }
                                            UpdateFnValue::Decimal(nv) => {
                                                if v.kind != VariableKind::Uint {
                                                    return Err(ContractError::Unauthorized {});
                                                    //todo: err
                                                }
                                                v.value = Some(
                                                    resolve_num_value_decimal(
                                                        deps,
                                                        env.clone(),
                                                        nv,
                                                        &vars,
                                                    )?
                                                    .to_string(),
                                                )
                                            }
                                            UpdateFnValue::Timestamp(nv) => {
                                                if v.kind != VariableKind::Int {
                                                    return Err(ContractError::Unauthorized {});
                                                    //todo: err
                                                }
                                                v.value = Some(
                                                    resolve_num_value_int(
                                                        deps,
                                                        env.clone(),
                                                        nv,
                                                        &vars,
                                                    )?
                                                    .to_string(),
                                                )
                                            }
                                            UpdateFnValue::BlockHeight(nv) => {
                                                if v.kind != VariableKind::Int {
                                                    return Err(ContractError::Unauthorized {});
                                                    //todo: err
                                                }
                                                v.value = Some(
                                                    resolve_num_value_int(
                                                        deps,
                                                        env.clone(),
                                                        nv,
                                                        &vars,
                                                    )?
                                                    .to_string(),
                                                )
                                            }
                                            UpdateFnValue::Bool(val) => {
                                                if v.kind != VariableKind::Bool {
                                                    return Err(ContractError::Unauthorized {});
                                                    //todo: err
                                                }
                                                v.value = Some(
                                                    resolve_ref_bool(
                                                        deps,
                                                        env.clone(),
                                                        val,
                                                        &vars,
                                                    )?
                                                    .to_string(),
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                            JobStatus::Cancelled => return Err(ContractError::Unauthorized {}),
                        }
                    }
                }
            }
            Variable::External(_) => {}
            Variable::Query(_) => {}
        }
    }
    Ok(vars)
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

pub fn is_recurring(vars: &Vec<Variable>) -> bool {
    for var in vars {
        match var {
            Variable::Static(v) => {
                if v.update_fn.is_some()
                    && (v.update_fn.as_ref().unwrap().on_success.is_some()
                        || v.update_fn.as_ref().unwrap().on_error.is_some())
                {
                    return true;
                }
            }
            Variable::External(v) => {
                if v.update_fn.is_some()
                    && (v.update_fn.as_ref().unwrap().on_success.is_some()
                        || v.update_fn.as_ref().unwrap().on_error.is_some())
                {
                    return true;
                }
            }
            Variable::Query(v) => {
                if v.update_fn.is_some()
                    && (v.update_fn.as_ref().unwrap().on_success.is_some()
                        || v.update_fn.as_ref().unwrap().on_error.is_some())
                {
                    return true;
                }
            }
        }
    }
    false
}

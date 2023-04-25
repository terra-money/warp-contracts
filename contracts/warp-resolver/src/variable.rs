use crate::condition::{
    resolve_num_value_decimal, resolve_num_value_int, resolve_num_value_uint,
    resolve_query_expr_bool, resolve_query_expr_decimal, resolve_query_expr_int,
    resolve_query_expr_string, resolve_query_expr_uint, resolve_ref_bool,
};
use crate::ContractError;
use cosmwasm_std::{CosmosMsg, Decimal256, Deps, Env, StdResult, Uint128, Uint256};
use std::str::FromStr;

use warp_protocol::controller::job::{ExternalInput, JobStatus};
use warp_protocol::resolver::variable::{UpdateFnValue, Variable, VariableKind};

pub fn hydrate_vars(
    deps: Deps,
    env: Env,
    vars: Vec<Variable>,
    external_inputs: Option<Vec<ExternalInput>>,
) -> Result<Vec<Variable>, ContractError> {
    let mut hydrated_vars = vec![];

    for var in vars {
        let hydrated_var = match var {
            Variable::Static(v) => Variable::Static(v),
            Variable::External(mut v) => {
                if v.reinitialize || v.value.is_none() {
                    match external_inputs {
                        None => {
                            if v.value.is_none() {
                                return Err(ContractError::HydrationError {
                                    msg: "External input value is none.".to_string(),
                                });
                            }
                            Variable::External(v)
                        }
                        Some(ref input) => {
                            let idx = input.iter().position(|i| i.name == v.name);
                            v.value = match idx {
                                None => {
                                    return Err(ContractError::HydrationError {
                                        msg: "External input variable not found.".to_string(),
                                    })
                                }
                                Some(i) => Some(input[i].input.clone()),
                            };
                            Variable::External(v)
                        }
                    }
                } else {
                    if v.value.is_none() {
                        return Err(ContractError::HydrationError {
                            msg: "External value is none.".to_string(),
                        });
                    }
                    Variable::External(v)
                }
            }
            Variable::Query(mut v) => {
                if v.reinitialize || v.value.is_none() {
                    match v.kind {
                        VariableKind::String => {
                            v.value = Some(
                                // \"$warp.variable\" => \"VALUE"\
                                resolve_query_expr_string(deps, env.clone(), v.init_fn.clone())?
                                    .to_string(),
                            )
                        }
                        VariableKind::Uint => {
                            v.value = Some(
                                resolve_query_expr_uint(deps, env.clone(), v.init_fn.clone())?
                                    .to_string(),
                            )
                        }
                        VariableKind::Int => {
                            v.value = Some(
                                resolve_query_expr_int(deps, env.clone(), v.init_fn.clone())?
                                    .to_string(),
                            )
                        }
                        VariableKind::Decimal => {
                            v.value = Some(
                                resolve_query_expr_decimal(deps, env.clone(), v.init_fn.clone())?
                                    .to_string(),
                            )
                        }
                        VariableKind::Timestamp => {
                            v.value = Some(
                                resolve_query_expr_int(deps, env.clone(), v.init_fn.clone())?
                                    .to_string(),
                            )
                        }
                        VariableKind::Bool => {
                            v.value = Some(
                                resolve_query_expr_bool(deps, env.clone(), v.init_fn.clone())?
                                    .to_string(),
                            )
                        }
                        VariableKind::Amount => {
                            v.value = Some(
                                resolve_query_expr_uint(deps, env.clone(), v.init_fn.clone())?
                                    .to_string(),
                            )
                        }
                        VariableKind::Asset => {
                            v.value = Some(
                                resolve_query_expr_string(deps, env.clone(), v.init_fn.clone())?
                                    .to_string(),
                            )
                        }
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
    for msg in msgs {
        let mut replaced_msg = msg.clone();
        for var in &vars {
            let (name, replacement) = match var {
                Variable::Static(v) => (
                    v.name.clone(),
                    match v.kind {
                        VariableKind::String => format!("\"{}\"", v.value),
                        VariableKind::Uint => format!("\"{}\"", v.value),
                        VariableKind::Int => v.value.clone(),
                        VariableKind::Decimal => format!("\"{}\"", v.value),
                        VariableKind::Timestamp => v.value.clone(),
                        VariableKind::Bool => v.value.clone(),
                        VariableKind::Amount => format!("\"{}\"", v.value),
                        VariableKind::Asset => format!("\"{}\"", v.value),
                    },
                ),
                Variable::External(v) => match v.value.clone() {
                    None => {
                        return Err(ContractError::HydrationError {
                            msg: "External msg value is none.".to_string(),
                        });
                    }
                    Some(val) => (
                        v.name.clone(),
                        match v.kind {
                            VariableKind::String => format!("\"{}\"", val),
                            VariableKind::Uint => format!("\"{}\"", val),
                            VariableKind::Int => val.clone(),
                            VariableKind::Decimal => format!("\"{}\"", val),
                            VariableKind::Timestamp => val.clone(),
                            VariableKind::Bool => val.clone(),
                            VariableKind::Amount => format!("\"{}\"", val),
                            VariableKind::Asset => format!("\"{}\"", val),
                        },
                    ),
                },
                Variable::Query(v) => match v.value.clone() {
                    None => {
                        return Err(ContractError::HydrationError {
                            msg: "Query msg value is none.".to_string(),
                        });
                    }
                    Some(val) => (
                        v.name.clone(),
                        match v.kind {
                            VariableKind::String => format!("\"{}\"", val),
                            VariableKind::Uint => format!("\"{}\"", val),
                            VariableKind::Int => val.clone(),
                            VariableKind::Decimal => format!("\"{}\"", val),
                            VariableKind::Timestamp => val.clone(),
                            VariableKind::Bool => val.clone(),
                            VariableKind::Amount => format!("\"{}\"", val),
                            VariableKind::Asset => format!("\"{}\"", val),
                        },
                    ),
                },
            };
            replaced_msg = msg.replace(&format!("\"$warp.variable.{}\"", name), &replacement);
            if replacement.contains("$warp.variable") {
                return Err(ContractError::HydrationError {
                    msg: "Attempt to inject warp variable.".to_string(),
                });
            }
        }
        parsed_msgs.push(serde_json_wasm::from_str::<CosmosMsg>(&replaced_msg)?)
    }

    Ok(parsed_msgs)
}

pub fn msgs_valid(msgs: &Vec<String>, vars: &Vec<Variable>) -> Result<bool, ContractError> {
    let mut parsed_msgs: Vec<CosmosMsg> = vec![];
    for msg in msgs {
        let mut replaced_msg = msg.clone();
        for var in vars {
            let (name, replacement) = match var {
                Variable::Static(v) => (
                    v.name.clone(),
                    match v.kind {
                        VariableKind::String => "\"test\"",
                        VariableKind::Uint => "\"0\"",
                        VariableKind::Int => "0",
                        VariableKind::Decimal => "\"0.0\"",
                        VariableKind::Timestamp => "0",
                        VariableKind::Bool => "true",
                        VariableKind::Amount => "\"0\"",
                        VariableKind::Asset => "\"test\"",
                    },
                ),
                Variable::External(v) => (
                    v.name.clone(),
                    match v.kind {
                        VariableKind::String => "\"test\"",
                        VariableKind::Uint => "\"0\"",
                        VariableKind::Int => "0",
                        VariableKind::Decimal => "\"0.0\"",
                        VariableKind::Timestamp => "0",
                        VariableKind::Bool => "true",
                        VariableKind::Amount => "\"0\"",
                        VariableKind::Asset => "\"test\"",
                    },
                ),
                Variable::Query(v) => (
                    v.name.clone(),
                    match v.kind {
                        VariableKind::String => "\"test\"",
                        VariableKind::Uint => "\"0\"",
                        VariableKind::Int => "0",
                        VariableKind::Decimal => "\"0.0\"",
                        VariableKind::Timestamp => "0",
                        VariableKind::Bool => "true",
                        VariableKind::Amount => "\"0\"",
                        VariableKind::Asset => "\"test\"",
                    },
                ),
            };
            replaced_msg = msg.replace(&format!("\"$warp.variable.{}\"", name), replacement);
            if replacement.contains("$warp.variable") {
                return Err(ContractError::HydrationError {
                    msg: "Attempt to inject warp variable.".to_string(),
                });
            }
        }
        parsed_msgs.push(serde_json_wasm::from_str::<CosmosMsg>(&replaced_msg)?)
    } //todo: check if msgs valid

    Ok(true)
}

pub fn apply_var_fn(
    deps: Deps,
    env: Env,
    vars: Vec<Variable>,
    status: JobStatus,
) -> Result<Vec<Variable>, ContractError> {
    let mut res = vec![];
    for var in vars.clone() {
        match var {
            Variable::Static(mut v) => {
                match v.update_fn.clone() {
                    None => (),
                    Some(update_fn) => match status {
                        JobStatus::Pending => {
                            return Err(ContractError::FunctionError {
                                msg: "Static job status pending.".to_string(),
                            })
                        }
                        JobStatus::Executed => match update_fn.on_success {
                            None => (),
                            Some(on_success) => match on_success {
                                UpdateFnValue::Uint(nv) => {
                                    if v.kind != VariableKind::Uint {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Uint function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = resolve_num_value_uint(deps, env.clone(), nv, &vars)?
                                        .to_string();
                                }
                                UpdateFnValue::Int(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Int function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                        .to_string();
                                }
                                UpdateFnValue::Decimal(nv) => {
                                    if v.kind != VariableKind::Decimal {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Decimal function mismatch.".to_string(),
                                        });
                                    }
                                    v.value =
                                        resolve_num_value_decimal(deps, env.clone(), nv, &vars)?
                                            .to_string();
                                }
                                UpdateFnValue::Timestamp(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Timestamp function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                        .to_string();
                                }
                                UpdateFnValue::BlockHeight(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static BlockHeight function mismatch."
                                                .to_string(),
                                        });
                                    }
                                    v.value = resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                        .to_string();
                                }
                                UpdateFnValue::Bool(val) => {
                                    if v.kind != VariableKind::Bool {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Bool function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = resolve_ref_bool(deps, env.clone(), val, &vars)?
                                        .to_string();
                                }
                            },
                        },
                        JobStatus::Failed => match update_fn.on_error {
                            None => (),
                            Some(on_success) => match on_success {
                                UpdateFnValue::Uint(nv) => {
                                    if v.kind != VariableKind::Uint {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Uint function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = resolve_num_value_uint(deps, env.clone(), nv, &vars)?
                                        .to_string();
                                }
                                UpdateFnValue::Int(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Int function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                        .to_string();
                                }
                                UpdateFnValue::Decimal(nv) => {
                                    if v.kind != VariableKind::Decimal {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Uint function mismatch.".to_string(),
                                        });
                                    }
                                    v.value =
                                        resolve_num_value_decimal(deps, env.clone(), nv, &vars)?
                                            .to_string()
                                }
                                UpdateFnValue::Timestamp(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Timestamp function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                        .to_string();
                                }
                                UpdateFnValue::BlockHeight(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static BlockHeight function mismatch."
                                                .to_string(),
                                        });
                                    }
                                    v.value = resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                        .to_string();
                                }
                                UpdateFnValue::Bool(val) => {
                                    if v.kind != VariableKind::Bool {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Bool function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = resolve_ref_bool(deps, env.clone(), val, &vars)?
                                        .to_string();
                                }
                            },
                        },
                        _ => {
                            return Err(ContractError::FunctionError {
                                msg: "Static status not supported.".to_string(),
                            })
                        }
                    },
                }
                res.push(Variable::Static(v));
            }
            Variable::External(mut v) => {
                match v.update_fn.clone() {
                    None => (),
                    Some(update_fn) => match status {
                        JobStatus::Pending => {
                            return Err(ContractError::FunctionError {
                                msg: "External job status pending.".to_string(),
                            })
                        }
                        JobStatus::Executed => match update_fn.on_success {
                            None => (),
                            Some(on_success) => match on_success {
                                UpdateFnValue::Uint(nv) => {
                                    if v.kind != VariableKind::Uint {
                                        return Err(ContractError::FunctionError {
                                            msg: "External Uint function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_uint(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::Int(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "External Int function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::Decimal(nv) => {
                                    if v.kind != VariableKind::Decimal {
                                        return Err(ContractError::FunctionError {
                                            msg: "External Decimal function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_decimal(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::Timestamp(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "External Timestamp function mismatch."
                                                .to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::BlockHeight(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "External BlockHeight function mismatch."
                                                .to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::Bool(val) => {
                                    if v.kind != VariableKind::Bool {
                                        return Err(ContractError::FunctionError {
                                            msg: "External Bool function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_ref_bool(deps, env.clone(), val, &vars)?
                                            .to_string(),
                                    )
                                }
                            },
                        },
                        JobStatus::Failed => match update_fn.on_error {
                            None => (),
                            Some(on_success) => match on_success {
                                UpdateFnValue::Uint(nv) => {
                                    if v.kind != VariableKind::Uint {
                                        return Err(ContractError::FunctionError {
                                            msg: "External Uint function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_uint(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::Int(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "External Int function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::Decimal(nv) => {
                                    if v.kind != VariableKind::Decimal {
                                        return Err(ContractError::FunctionError {
                                            msg: "External Decimal function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_decimal(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::Timestamp(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "External Timestamp function mismatch."
                                                .to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::BlockHeight(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "External BlockHeight function mismatch."
                                                .to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::Bool(val) => {
                                    if v.kind != VariableKind::Bool {
                                        return Err(ContractError::FunctionError {
                                            msg: "External Bool function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_ref_bool(deps, env.clone(), val, &vars)?
                                            .to_string(),
                                    )
                                }
                            },
                        },
                        _ => {
                            return Err(ContractError::FunctionError {
                                msg: "External status not supported.".to_string(),
                            })
                        }
                    },
                }
                res.push(Variable::External(v));
            }
            Variable::Query(mut v) => {
                match v.update_fn.clone() {
                    None => (),
                    Some(update_fn) => match status {
                        JobStatus::Pending => {
                            return Err(ContractError::FunctionError {
                                msg: "Query job status pending.".to_string(),
                            })
                        }
                        JobStatus::Executed => match update_fn.on_success {
                            None => (),
                            Some(on_success) => match on_success {
                                UpdateFnValue::Uint(nv) => {
                                    if v.kind != VariableKind::Uint {
                                        return Err(ContractError::FunctionError {
                                            msg: "Query Uint function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_uint(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::Int(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "Query Int function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::Decimal(nv) => {
                                    if v.kind != VariableKind::Decimal {
                                        return Err(ContractError::FunctionError {
                                            msg: "Query Decimal function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_decimal(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::Timestamp(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "Query Timestamp function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::BlockHeight(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "Query Blockheighht function mismatch."
                                                .to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::Bool(val) => {
                                    if v.kind != VariableKind::Bool {
                                        return Err(ContractError::FunctionError {
                                            msg: "Query Bool function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_ref_bool(deps, env.clone(), val, &vars)?
                                            .to_string(),
                                    )
                                }
                            },
                        },
                        JobStatus::Failed => match update_fn.on_error {
                            None => (),
                            Some(on_success) => match on_success {
                                UpdateFnValue::Uint(nv) => {
                                    if v.kind != VariableKind::Uint {
                                        return Err(ContractError::FunctionError {
                                            msg: "Query Uint function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_uint(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::Int(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "Query Int function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::Decimal(nv) => {
                                    if v.kind != VariableKind::Decimal {
                                        return Err(ContractError::FunctionError {
                                            msg: "Query Decimal function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_decimal(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::Timestamp(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "Query Timestamp function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::BlockHeight(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "Query BlockHeight function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    )
                                }
                                UpdateFnValue::Bool(val) => {
                                    if v.kind != VariableKind::Bool {
                                        return Err(ContractError::FunctionError {
                                            msg: "Query Bool function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_ref_bool(deps, env.clone(), val, &vars)?
                                            .to_string(),
                                    )
                                }
                            },
                        },
                        _ => {
                            return Err(ContractError::FunctionError {
                                msg: "Query status not supported.".to_string(),
                            })
                        }
                    },
                }
                res.push(Variable::Query(v));
            }
        }
    }
    Ok(res)
}

pub fn get_var(name: String, vars: &Vec<Variable>) -> Result<&Variable, ContractError> {
    for var in vars {
        let n = match var {
            Variable::Static(v) => v.name.clone(),
            Variable::External(v) => v.name.clone(),
            Variable::Query(v) => v.name.clone(),
        };
        if format!("$warp.variable.{}", n) == name {
            return Ok(var);
        }
    }
    Err(ContractError::VariableNotFound { name })
}

pub fn has_duplicates(vars: &Vec<Variable>) -> bool {
    for i in 0..vars.len() {
        for j in i..vars.len() {
            if i != j {
                match vars[i].clone() {
                    Variable::Static(vari) => match vars[j].clone() {
                        Variable::Static(varj) => {
                            if vari.name == varj.name {
                                return true;
                            }
                        }
                        Variable::External(varj) => {
                            if vari.name == varj.name {
                                return true;
                            }
                        }
                        Variable::Query(varj) => {
                            if vari.name == varj.name {
                                return true;
                            }
                        }
                    },
                    Variable::External(vari) => match vars[j].clone() {
                        Variable::Static(varj) => {
                            if vari.name == varj.name {
                                return true;
                            }
                        }
                        Variable::External(varj) => {
                            if vari.name == varj.name {
                                return true;
                            }
                        }
                        Variable::Query(varj) => {
                            if vari.name == varj.name {
                                return true;
                            }
                        }
                    },
                    Variable::Query(vari) => match vars[j].clone() {
                        Variable::Static(varj) => {
                            if vari.name == varj.name {
                                return true;
                            }
                        }
                        Variable::External(varj) => {
                            if vari.name == varj.name {
                                return true;
                            }
                        }
                        Variable::Query(varj) => {
                            if vari.name == varj.name {
                                return true;
                            }
                        }
                    },
                }
            }
        }
    }
    false
}

pub fn string_vars_in_vector(vars: &Vec<Variable>, s: &String) -> bool {
    let mut s = s.to_owned();
    for var in vars {
        let name = get_var_name(var);
        s = s.replace(format!("$warp.variable.{}", name).as_str(), "VAR_CHECKED")
    }
    if s.contains("$warp.variable.") {
        return false;
    }
    true
}

pub fn all_vector_vars_present(vars: &Vec<Variable>, s: String) -> bool {
    for var in vars {
        let name = get_var_name(var);
        if !s.contains(format!("$warp.variable.{}", name.as_str()).as_str()) {
            return false;
        }
    }
    true
}

fn get_var_name(var: &Variable) -> String {
    match var.clone() {
        Variable::Static(v) => v.name.clone(),
        Variable::External(v) => v.name.clone(),
        Variable::Query(v) => v.name.clone(),
    }
}

pub fn vars_valid(vars: &Vec<Variable>) -> bool {
    for var in vars {
        match var {
            Variable::Static(v) => match v.kind {
                VariableKind::String => {}
                VariableKind::Uint => {
                    if Uint256::from_str(&v.value).is_err() {
                        return false;
                    }
                }
                VariableKind::Int => {
                    if i128::from_str(&v.value).is_err() {
                        return false;
                    }
                }
                VariableKind::Decimal => {
                    if Decimal256::from_str(&v.value).is_err() {
                        return false;
                    }
                }
                VariableKind::Timestamp => {
                    if i128::from_str(&v.value).is_err() {
                        return false;
                    }
                }
                VariableKind::Bool => {
                    if bool::from_str(&v.value).is_err() {
                        return false;
                    }
                }
                VariableKind::Amount => {
                    if Uint128::from_str(&v.value).is_err() {
                        return false;
                    }
                }
                VariableKind::Asset => {
                    if v.value.is_empty() {
                        return false;
                    }
                }
            },
            Variable::External(v) => {
                if v.reinitialize && v.update_fn.is_some() {
                    return false;
                }

                if let Some(val) = v.value.clone() {
                    match v.kind {
                        VariableKind::String => {}
                        VariableKind::Uint => {
                            if Uint256::from_str(&val).is_err() {
                                return false;
                            }
                        }
                        VariableKind::Int => {
                            if i128::from_str(&val).is_err() {
                                return false;
                            }
                        }
                        VariableKind::Decimal => {
                            if Decimal256::from_str(&val).is_err() {
                                return false;
                            }
                        }
                        VariableKind::Timestamp => {
                            if i128::from_str(&val).is_err() {
                                return false;
                            }
                        }
                        VariableKind::Bool => {
                            if bool::from_str(&val).is_err() {
                                return false;
                            }
                        }
                        VariableKind::Amount => {
                            if Uint128::from_str(&val).is_err() {
                                return false;
                            }
                        }
                        VariableKind::Asset => {
                            if val.is_empty() {
                                return false;
                            }
                        }
                    }
                }
            }
            Variable::Query(v) => {
                if v.reinitialize && v.update_fn.is_some() {
                    return false;
                }
                if let Some(val) = v.value.clone() {
                    match v.kind {
                        VariableKind::String => {}
                        VariableKind::Uint => {
                            if Uint256::from_str(&val).is_err() {
                                return false;
                            }
                        }
                        VariableKind::Int => {
                            if i128::from_str(&val).is_err() {
                                return false;
                            }
                        }
                        VariableKind::Decimal => {
                            if Decimal256::from_str(&val).is_err() {
                                return false;
                            }
                        }
                        VariableKind::Timestamp => {
                            if i128::from_str(&val).is_err() {
                                return false;
                            }
                        }
                        VariableKind::Bool => {
                            if bool::from_str(&val).is_err() {
                                return false;
                            }
                        }
                        VariableKind::Amount => {
                            if Uint128::from_str(&val).is_err() {
                                return false;
                            }
                        }
                        VariableKind::Asset => {
                            if val.is_empty() {
                                return false;
                            }
                        }
                    }
                }
            }
        }
    }
    true
}

pub fn validate_vars_and_msgs(vars: Vec<Variable>, cond_string: String, msg_string: String) -> Result<bool, ContractError> {
    if !vars_valid(&vars) {
        return Err(ContractError::InvalidVariables {});
    }

    if has_duplicates(&vars) {
        return Err(ContractError::VariablesContainDuplicates {});
    }

    if !(string_vars_in_vector(&vars, &cond_string)
        && string_vars_in_vector(&vars, &msg_string))
    {
        return Err(ContractError::VariablesMissingFromVector {});
    }

    if !all_vector_vars_present(&vars, format!("{}{}", cond_string, msg_string)) {
        return Err(ContractError::ExcessVariablesInVector {});
    }

    if !msgs_valid(&serde_json_wasm::from_str(&msg_string)?, &vars)? {
        return Err(ContractError::MsgError {
            msg: "msgs are invalid".to_string(),
        });
    }

    Ok(true)
}

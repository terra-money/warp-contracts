use crate::util::condition::{
    resolve_num_value_decimal, resolve_num_value_int, resolve_num_value_uint,
    resolve_query_expr_bool, resolve_query_expr_decimal, resolve_query_expr_int,
    resolve_query_expr_string, resolve_query_expr_uint, resolve_ref_bool,
};
use crate::ContractError;
use controller::account::WarpMsg;
use cosmwasm_schema::serde::de::DeserializeOwned;
use cosmwasm_schema::serde::Serialize;
use cosmwasm_std::{
    Binary, CosmosMsg, Decimal256, Deps, Env, QueryRequest, Uint128, Uint256, WasmQuery,
};
use std::str::FromStr;

use controller::job::{ExternalInput, JobStatus};
use resolver::variable::{FnValue, QueryExpr, Variable, VariableKind};

use super::condition::resolve_string_value;

pub fn hydrate_vars(
    deps: Deps,
    env: Env,
    vars: Vec<Variable>,
    external_inputs: Option<Vec<ExternalInput>>,
    warp_account_addr: Option<String>,
) -> Result<Vec<Variable>, ContractError> {
    let mut hydrated_vars = vec![];

    for var in vars {
        let hydrated_var = match var {
            Variable::Static(mut v) => {
                if v.reinitialize || v.value.is_none() {
                    match v.kind {
                        VariableKind::Uint => match v.init_fn.clone() {
                            FnValue::Uint(val) => {
                                v.value = Some(replace_in_string(
                                    resolve_num_value_uint(deps, env.clone(), val, &hydrated_vars)?
                                        .to_string(),
                                    &hydrated_vars,
                                )?)
                            }
                            _ => {
                                return Err(ContractError::HydrationError {
                                    msg: "Variable init_fn is not of type FnValue::Uint."
                                        .to_string(),
                                })
                            }
                        },
                        VariableKind::Int => match v.init_fn.clone() {
                            FnValue::Int(val) => {
                                v.value = Some(replace_in_string(
                                    resolve_num_value_int(deps, env.clone(), val, &hydrated_vars)?
                                        .to_string(),
                                    &hydrated_vars,
                                )?)
                            }
                            _ => {
                                return Err(ContractError::HydrationError {
                                    msg: "Variable init_fn is not of type FnValue::Int."
                                        .to_string(),
                                })
                            }
                        },
                        VariableKind::Decimal => match v.init_fn.clone() {
                            FnValue::Decimal(val) => {
                                v.value = Some(replace_in_string(
                                    resolve_num_value_decimal(
                                        deps,
                                        env.clone(),
                                        val,
                                        &hydrated_vars,
                                    )?
                                    .to_string(),
                                    &hydrated_vars,
                                )?)
                            }
                            _ => {
                                return Err(ContractError::HydrationError {
                                    msg: "Variable init_fn is not of type FnValue::Decimal."
                                        .to_string(),
                                })
                            }
                        },
                        VariableKind::Timestamp => match v.init_fn.clone() {
                            FnValue::Timestamp(val) => {
                                v.value = Some(replace_in_string(
                                    resolve_num_value_int(deps, env.clone(), val, &hydrated_vars)?
                                        .to_string(),
                                    &hydrated_vars,
                                )?)
                            }
                            _ => {
                                return Err(ContractError::HydrationError {
                                    msg: "Variable init_fn is not of type FnValue::Timestamp."
                                        .to_string(),
                                })
                            }
                        },
                        VariableKind::Bool => match v.init_fn.clone() {
                            FnValue::Bool(val) => {
                                v.value = Some(replace_in_string(
                                    resolve_ref_bool(deps, env.clone(), val, &hydrated_vars)?
                                        .to_string(),
                                    &hydrated_vars,
                                )?)
                            }
                            _ => {
                                return Err(ContractError::HydrationError {
                                    msg: "Variable init_fn is not of type FnValue::Bool."
                                        .to_string(),
                                })
                            }
                        },
                        VariableKind::Amount => match v.init_fn.clone() {
                            FnValue::Uint(val) => {
                                v.value = Some(replace_in_string(
                                    resolve_num_value_uint(deps, env.clone(), val, &hydrated_vars)?
                                        .to_string(),
                                    &hydrated_vars,
                                )?)
                            }
                            _ => {
                                return Err(ContractError::HydrationError {
                                    msg: "Variable init_fn is not of type FnValue::Uint."
                                        .to_string(),
                                })
                            }
                        },
                        VariableKind::String => match v.init_fn.clone() {
                            FnValue::String(val) => {
                                v.value = Some(replace_in_string(
                                    resolve_string_value(
                                        deps,
                                        env.clone(),
                                        val,
                                        &hydrated_vars,
                                        warp_account_addr.clone(),
                                    )?,
                                    &hydrated_vars,
                                )?)
                            }
                            _ => {
                                return Err(ContractError::HydrationError {
                                    msg: "1Variable init_fn is not of type FnValue::String."
                                        .to_string(),
                                })
                            }
                        },
                        VariableKind::Asset => match v.init_fn.clone() {
                            FnValue::String(val) => {
                                v.value = Some(replace_in_string(
                                    resolve_string_value(
                                        deps,
                                        env.clone(),
                                        val,
                                        &hydrated_vars,
                                        warp_account_addr.clone(),
                                    )?,
                                    &hydrated_vars,
                                )?)
                            }
                            _ => {
                                return Err(ContractError::HydrationError {
                                    msg: "Variable init_fn is not of type FnValue::String."
                                        .to_string(),
                                })
                            }
                        },
                        VariableKind::Json => match v.init_fn.clone() {
                            FnValue::String(val) => {
                                v.value = Some(replace_in_string(
                                    resolve_string_value(
                                        deps,
                                        env.clone(),
                                        val,
                                        &hydrated_vars,
                                        warp_account_addr.clone(),
                                    )?,
                                    &hydrated_vars,
                                )?)
                            }
                            _ => {
                                return Err(ContractError::HydrationError {
                                    msg: "Variable init_fn is not of type FnValue::String."
                                        .to_string(),
                                });
                            }
                        },
                    }
                }
                if v.value.is_none() {
                    return Err(ContractError::Unauthorized {});
                }
                Variable::Static(v)
            }
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
                    v.init_fn = replace_references(v.init_fn, &hydrated_vars)?;

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
                        VariableKind::Json => {
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

pub fn hydrate_msgs(msgs: String, vars: Vec<Variable>) -> Result<Vec<WarpMsg>, ContractError> {
    let mut replaced_msgs = msgs;
    for var in &vars {
        let (name, replacement) = get_replacement_in_struct(var)?;
        replaced_msgs =
            replaced_msgs.replace(&format!("\"$warp.variable.{}\"", name), &replacement);
        if replacement.contains("$warp.variable") {
            return Err(ContractError::HydrationError {
                msg: "Attempt to inject warp variable.".to_string(),
            });
        }
    }

    match serde_json_wasm::from_str::<Vec<WarpMsg>>(&replaced_msgs) {
        Ok(msgs) => Ok(msgs),

        // fallback to legacy flow
        Err(_) => {
            let msgs = serde_json_wasm::from_str::<Vec<CosmosMsg>>(&replaced_msgs)
                .unwrap()
                .into_iter()
                .map(WarpMsg::Generic)
                .collect();

            Ok(msgs)
        }
    }
}

fn get_replacement_in_struct(var: &Variable) -> Result<(String, String), ContractError> {
    let (name, replacement) = match var {
        Variable::Static(v) => match v.value.clone() {
            None => {
                return Err(ContractError::HydrationError {
                    msg: "Static msg value is none.".to_string(),
                });
            }
            Some(val) => (v.name.clone(), {
                match v.kind {
                    VariableKind::Uint => format!(
                        "\"{}\"",
                        match v.encode {
                            true => {
                                base64::encode(val)
                            }
                            false => val,
                        }
                    ),
                    VariableKind::Int => match v.encode {
                        true => {
                            format!("\"{}\"", base64::encode(val))
                        }
                        false => val,
                    },
                    VariableKind::Decimal => format!(
                        "\"{}\"",
                        match v.encode {
                            true => {
                                base64::encode(val)
                            }
                            false => val,
                        }
                    ),
                    VariableKind::Timestamp => match v.encode {
                        true => {
                            format!("\"{}\"", base64::encode(val))
                        }
                        false => val,
                    },
                    VariableKind::Bool => match v.encode {
                        true => {
                            format!("\"{}\"", base64::encode(val))
                        }
                        false => val,
                    },
                    VariableKind::Amount => format!(
                        "\"{}\"",
                        match v.encode {
                            true => {
                                base64::encode(val)
                            }
                            false => val,
                        }
                    ),
                    VariableKind::String => format!(
                        "\"{}\"",
                        match v.encode {
                            true => {
                                base64::encode(val)
                            }
                            false => val,
                        }
                    ),
                    VariableKind::Asset => format!(
                        "\"{}\"",
                        match v.encode {
                            true => {
                                base64::encode(val)
                            }
                            false => val,
                        }
                    ),
                    VariableKind::Json => match v.encode {
                        true => {
                            format!("\"{}\"", base64::encode(val))
                        }
                        false => val,
                    },
                }
            }),
        },
        Variable::External(v) => match v.value.clone() {
            None => {
                return Err(ContractError::HydrationError {
                    msg: "External msg value is none.".to_string(),
                });
            }
            Some(val) => (v.name.clone(), {
                match v.kind {
                    VariableKind::String => format!(
                        "\"{}\"",
                        match v.encode {
                            true => {
                                base64::encode(val)
                            }
                            false => val,
                        }
                    ),
                    VariableKind::Uint => format!(
                        "\"{}\"",
                        match v.encode {
                            true => {
                                base64::encode(val)
                            }
                            false => val,
                        }
                    ),
                    VariableKind::Int => match v.encode {
                        true => {
                            format!("\"{}\"", base64::encode(val))
                        }
                        false => val,
                    },
                    VariableKind::Decimal => format!(
                        "\"{}\"",
                        match v.encode {
                            true => {
                                base64::encode(val)
                            }
                            false => val,
                        }
                    ),
                    VariableKind::Timestamp => match v.encode {
                        true => {
                            format!("\"{}\"", base64::encode(val))
                        }
                        false => val,
                    },
                    VariableKind::Bool => match v.encode {
                        true => {
                            format!("\"{}\"", base64::encode(val))
                        }
                        false => val,
                    },
                    VariableKind::Amount => format!(
                        "\"{}\"",
                        match v.encode {
                            true => {
                                base64::encode(val)
                            }
                            false => val,
                        }
                    ),
                    VariableKind::Asset => format!(
                        "\"{}\"",
                        match v.encode {
                            true => {
                                base64::encode(val)
                            }
                            false => val,
                        }
                    ),
                    VariableKind::Json => match v.encode {
                        true => {
                            format!("\"{}\"", base64::encode(val))
                        }
                        false => val,
                    },
                }
            }),
        },
        Variable::Query(v) => match v.value.clone() {
            None => {
                return Err(ContractError::HydrationError {
                    msg: "Query msg value is none.".to_string(),
                });
            }
            Some(val) => (v.name.clone(), {
                match v.kind {
                    VariableKind::Uint => format!(
                        "\"{}\"",
                        match v.encode {
                            true => {
                                base64::encode(val)
                            }
                            false => val,
                        }
                    ),
                    VariableKind::Int => match v.encode {
                        true => {
                            format!("\"{}\"", base64::encode(val))
                        }
                        false => val,
                    },
                    VariableKind::Decimal => format!(
                        "\"{}\"",
                        match v.encode {
                            true => {
                                base64::encode(val)
                            }
                            false => val,
                        }
                    ),
                    VariableKind::Timestamp => match v.encode {
                        true => {
                            format!("\"{}\"", base64::encode(val))
                        }
                        false => val,
                    },
                    VariableKind::Bool => match v.encode {
                        true => {
                            format!("\"{}\"", base64::encode(val))
                        }
                        false => val,
                    },
                    VariableKind::Amount => format!(
                        "\"{}\"",
                        match v.encode {
                            true => {
                                base64::encode(val)
                            }
                            false => val,
                        }
                    ),
                    VariableKind::String => format!(
                        "\"{}\"",
                        match v.encode {
                            true => {
                                base64::encode(val)
                            }
                            false => val,
                        }
                    ),
                    VariableKind::Asset => format!(
                        "\"{}\"",
                        match v.encode {
                            true => {
                                base64::encode(val)
                            }
                            false => val,
                        }
                    ),
                    VariableKind::Json => match v.encode {
                        true => {
                            format!("\"{}\"", base64::encode(val))
                        }
                        false => val,
                    },
                }
            }),
        },
    };

    Ok((name, replacement))
}

fn get_replacement_in_string(var: &Variable) -> Result<(String, String), ContractError> {
    let (name, replacement) = match var {
        Variable::Static(v) => match v.value.clone() {
            None => {
                return Err(ContractError::HydrationError {
                    msg: "Static msg value is none.".to_string(),
                });
            }
            Some(val) => (
                v.name.clone(),
                match v.encode {
                    true => base64::encode(val),
                    false => val,
                },
            ),
        },
        Variable::External(v) => match v.value.clone() {
            None => {
                return Err(ContractError::HydrationError {
                    msg: "External msg value is none.".to_string(),
                });
            }
            Some(val) => (
                v.name.clone(),
                match v.encode {
                    true => base64::encode(val),
                    false => val,
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
                match v.encode {
                    true => base64::encode(val),
                    false => val,
                },
            ),
        },
    };

    Ok((name, replacement))
}

fn replace_references(mut expr: QueryExpr, vars: &[Variable]) -> Result<QueryExpr, ContractError> {
    match &mut expr.query {
        QueryRequest::Wasm(WasmQuery::Smart { msg, contract_addr }) => {
            *msg = replace_in_binary(msg, vars)?;
            *contract_addr = replace_in_string(contract_addr.to_string(), vars)?;
        }
        QueryRequest::Wasm(WasmQuery::Raw { key, contract_addr }) => {
            *key = replace_in_binary(key, vars)?;
            *contract_addr = replace_in_string(contract_addr.to_string(), vars)?;
        }
        QueryRequest::Custom(str) => {
            *str = replace_in_struct_string(str.to_string(), vars)?;
        }
        _ => {
            expr.query = replace_in_struct(&expr.query, vars)?;
        }
    }

    Ok(expr)
}

fn replace_in_binary(binary_str: &Binary, vars: &[Variable]) -> Result<Binary, ContractError> {
    let decoded =
        base64::decode(binary_str.to_string()).map_err(|_| ContractError::HydrationError {
            msg: "Failed to decode Base64.".to_string(),
        })?;
    let decoded_string = String::from_utf8(decoded).map_err(|_| ContractError::HydrationError {
        msg: "Failed to convert from UTF8.".to_string(),
    })?;

    let updated_string = replace_in_struct_string(decoded_string, vars)?;

    Ok(Binary::from(updated_string.as_bytes()))
}

fn replace_in_struct<T: Serialize + DeserializeOwned>(
    struct_val: &T,
    vars: &[Variable],
) -> Result<T, ContractError> {
    let struct_as_json =
        serde_json_wasm::to_string(&struct_val).map_err(|_| ContractError::HydrationError {
            msg: "Failed to convert struct to JSON.".to_string(),
        })?;
    let updated_struct_as_json = replace_in_struct_string(struct_as_json, vars)?;
    
    let replaced_value = serde_json_wasm::from_str(&updated_struct_as_json).map_err(|_| ContractError::HydrationError {
        msg: "Failed to convert JSON back to struct.".to_string(),
    })?;

    Ok(replaced_value)
}

fn replace_in_struct_string(value: String, vars: &[Variable]) -> Result<String, ContractError> {
    let mut replaced_value = value;

    for var in vars {
        let (name, replacement) = get_replacement_in_struct(var)?;
        replaced_value =
            replaced_value.replace(&format!("\"$warp.variable.{}\"", name), &replacement);
    }

    Ok(replaced_value)
}

fn replace_in_string(value: String, vars: &[Variable]) -> Result<String, ContractError> {
    let mut replaced_value = value;

    for var in vars {
        let (name, replacement) = get_replacement_in_string(var)?;
        replaced_value = replaced_value.replace(&format!("$warp.variable.{}", name), &replacement);
    }

    Ok(replaced_value)
}

pub fn msgs_valid(msgs: &str, vars: &Vec<Variable>) -> Result<bool, ContractError> {
    let mut replaced_msgs = msgs.to_owned();
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
                    VariableKind::Json => "true",
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
                    VariableKind::Json => "true",
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
                    VariableKind::Json => "true",
                },
            ),
        };
        if replacement.contains("$warp.variable") {
            return Err(ContractError::HydrationError {
                msg: "Attempt to inject warp variable.".to_string(),
            });
        }
        let replacement_with_encode = match var {
            Variable::Static(v) => match v.encode {
                true => format!("\"{}\"", base64::encode(replacement)),
                false => replacement.to_string(),
            },
            Variable::External(v) => match v.encode {
                true => format!("\"{}\"", base64::encode(replacement)),
                false => replacement.to_string(),
            },
            Variable::Query(v) => match v.encode {
                true => format!("\"{}\"", base64::encode(replacement)),
                false => replacement.to_string(),
            },
        };
        replaced_msgs = replaced_msgs.replace(
            &format!("\"$warp.variable.{}\"", name),
            &replacement_with_encode,
        );
    }

    let _msgs = serde_json_wasm::from_str::<Vec<WarpMsg>>(&replaced_msgs)?;

    Ok(true)
}

pub fn apply_var_fn(
    deps: Deps,
    env: Env,
    vars: Vec<Variable>,
    status: JobStatus,
    warp_account_addr: Option<String>,
) -> Result<String, ContractError> {
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
                                FnValue::Uint(nv) => {
                                    if v.kind != VariableKind::Uint {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Uint function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_uint(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    );
                                }
                                FnValue::Int(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Int function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    );
                                }
                                FnValue::Decimal(nv) => {
                                    if v.kind != VariableKind::Decimal {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Decimal function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_decimal(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    );
                                }
                                FnValue::Timestamp(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Timestamp function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    );
                                }
                                FnValue::BlockHeight(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static BlockHeight function mismatch."
                                                .to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    );
                                }
                                FnValue::Bool(val) => {
                                    if v.kind != VariableKind::Bool {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Bool function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_ref_bool(deps, env.clone(), val, &vars)?
                                            .to_string(),
                                    );
                                }
                                FnValue::String(val) => {
                                    if v.kind != VariableKind::String {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static String function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_string_value(
                                            deps,
                                            env.clone(),
                                            val,
                                            &vars,
                                            warp_account_addr.clone(),
                                        )?
                                        .to_string(),
                                    );
                                }
                            },
                        },
                        JobStatus::Failed => match update_fn.on_error {
                            None => (),
                            Some(on_success) => match on_success {
                                FnValue::Uint(nv) => {
                                    if v.kind != VariableKind::Uint {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Uint function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_uint(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    );
                                }
                                FnValue::Int(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Int function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    );
                                }
                                FnValue::Decimal(nv) => {
                                    if v.kind != VariableKind::Decimal {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Uint function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_decimal(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    );
                                }
                                FnValue::Timestamp(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Timestamp function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    );
                                }
                                FnValue::BlockHeight(nv) => {
                                    if v.kind != VariableKind::Int {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static BlockHeight function mismatch."
                                                .to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_num_value_int(deps, env.clone(), nv, &vars)?
                                            .to_string(),
                                    );
                                }
                                FnValue::Bool(val) => {
                                    if v.kind != VariableKind::Bool {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static Bool function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_ref_bool(deps, env.clone(), val, &vars)?
                                            .to_string(),
                                    );
                                }
                                FnValue::String(val) => {
                                    if v.kind != VariableKind::String {
                                        return Err(ContractError::FunctionError {
                                            msg: "Static String function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_string_value(
                                            deps,
                                            env.clone(),
                                            val,
                                            &vars,
                                            warp_account_addr.clone(),
                                        )?
                                        .to_string(),
                                    );
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
                                FnValue::Uint(nv) => {
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
                                FnValue::Int(nv) => {
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
                                FnValue::Decimal(nv) => {
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
                                FnValue::Timestamp(nv) => {
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
                                FnValue::BlockHeight(nv) => {
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
                                FnValue::Bool(val) => {
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
                                FnValue::String(val) => {
                                    if v.kind != VariableKind::String {
                                        return Err(ContractError::FunctionError {
                                            msg: "External String function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_string_value(
                                            deps,
                                            env.clone(),
                                            val,
                                            &vars,
                                            warp_account_addr.clone(),
                                        )?
                                        .to_string(),
                                    )
                                }
                            },
                        },
                        JobStatus::Failed => match update_fn.on_error {
                            None => (),
                            Some(on_success) => match on_success {
                                FnValue::Uint(nv) => {
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
                                FnValue::Int(nv) => {
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
                                FnValue::Decimal(nv) => {
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
                                FnValue::Timestamp(nv) => {
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
                                FnValue::BlockHeight(nv) => {
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
                                FnValue::Bool(val) => {
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
                                FnValue::String(val) => {
                                    if v.kind != VariableKind::String {
                                        return Err(ContractError::FunctionError {
                                            msg: "External String function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_string_value(
                                            deps,
                                            env.clone(),
                                            val,
                                            &vars,
                                            warp_account_addr.clone(),
                                        )?
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
                                FnValue::Uint(nv) => {
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
                                FnValue::Int(nv) => {
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
                                FnValue::Decimal(nv) => {
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
                                FnValue::Timestamp(nv) => {
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
                                FnValue::BlockHeight(nv) => {
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
                                FnValue::Bool(val) => {
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
                                FnValue::String(val) => {
                                    if v.kind != VariableKind::String {
                                        return Err(ContractError::FunctionError {
                                            msg: "Query String function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_string_value(
                                            deps,
                                            env.clone(),
                                            val,
                                            &vars,
                                            warp_account_addr.clone(),
                                        )?
                                        .to_string(),
                                    )
                                }
                            },
                        },
                        JobStatus::Failed => match update_fn.on_error {
                            None => (),
                            Some(on_success) => match on_success {
                                FnValue::Uint(nv) => {
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
                                FnValue::Int(nv) => {
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
                                FnValue::Decimal(nv) => {
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
                                FnValue::Timestamp(nv) => {
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
                                FnValue::BlockHeight(nv) => {
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
                                FnValue::Bool(val) => {
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
                                FnValue::String(val) => {
                                    if v.kind != VariableKind::String {
                                        return Err(ContractError::FunctionError {
                                            msg: "Query String function mismatch.".to_string(),
                                        });
                                    }
                                    v.value = Some(
                                        resolve_string_value(
                                            deps,
                                            env.clone(),
                                            val,
                                            &vars,
                                            warp_account_addr.clone(),
                                        )?
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
    Ok(serde_json_wasm::to_string(&res)?)
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

fn get_var_name(var: &Variable) -> String {
    match var.clone() {
        Variable::Static(v) => v.name,
        Variable::External(v) => v.name,
        Variable::Query(v) => v.name,
    }
}

pub fn vars_valid(vars: &Vec<Variable>) -> bool {
    for var in vars {
        match var {
            Variable::Static(v) => {
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
                        VariableKind::Json => {}
                    }
                }
            }
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
                        VariableKind::Json => {}
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
                        VariableKind::Json => {}
                    }
                }
            }
        }
    }
    true
}

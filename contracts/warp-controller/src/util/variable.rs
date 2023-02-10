use crate::util::condition::{
    resolve_num_value_decimal, resolve_num_value_int, resolve_num_value_uint,
    resolve_query_expr_bool, resolve_query_expr_decimal, resolve_query_expr_int,
    resolve_query_expr_string, resolve_query_expr_uint, resolve_ref_bool,
};
use crate::ContractError;
use cosmwasm_std::{CosmosMsg, Deps, Env};
use warp_protocol::controller::condition::Condition;

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
            Variable::Static(v) => Variable::Static(v),
            Variable::External(mut v) => {
                if v.reinitialize || v.value.is_none() {
                    match external_inputs {
                        None => {
                            if v.value.is_none() {
                                return Err(ContractError::HydrationError { msg: "External input value is none.".to_string() });
                            }
                            Variable::External(v)
                        }
                        Some(ref input) => {
                            let idx = input.iter().position(|i| i.name == v.name);
                            v.value = match idx {
                                None => return Err(ContractError::HydrationError { msg: "External input variable not found.".to_string() }),
                                Some(i) => Some(input[i].input.clone()),
                            };
                            Variable::External(v)
                        }
                    }
                } else {
                    if v.value.is_none() {
                        return Err(ContractError::HydrationError { msg: "External value is none.".to_string() });
                    }
                    Variable::External(v)
                }
            }
            Variable::Query(mut v) => {
                if v.reinitialize || v.value.is_none() {
                    match v.kind {
                        VariableKind::String => {
                            v.value = Some(format!(
                                "\"{}\"", // \"$warp.variable\" => \"VALUE"\
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
                                "{}", //\"$warp.variable\" => VALUE
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
                Variable::Static(v) => (v.name.clone(), v.value.clone()),
                Variable::External(v) => {
                    match v.value.clone() {
                        None => {
                            return Err(ContractError::HydrationError { msg: "External msg value is none.".to_string() });
                        }
                        Some(val) => (v.name.clone(), val),
                    }
                }
                Variable::Query(v) => {
                    match v.value.clone() {
                        None => {
                            return Err(ContractError::HydrationError { msg: "Query msg value is none.".to_string() });
                        }
                        Some(val) => (v.name.clone(), val),
                    }
                }
            };
            msg = msg.replace(&format!("\"$warp.variable.{}\"", name), &replacement);
            if replacement.contains("$warp.variable") {
                return Err(ContractError::HydrationError { msg: "Attempt to inject warp variable.".to_string() });
            }
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
    let mut res = vec![];
    for var in vars.clone() {
        match var {
            Variable::Static(mut v) => {
                match v.update_fn.clone() {
                    None => (),
                    Some(update_fn) => {
                        match status {
                            JobStatus::Pending => return Err(ContractError::FunctionError { msg: "Static job status pending.".to_string() }),
                            JobStatus::Executed => {
                                match update_fn.on_success {
                                    None => (),
                                    Some(on_success) => {
                                        match on_success {
                                            UpdateFnValue::Uint(nv) => {
                                                if v.kind != VariableKind::Uint {
                                                    return Err(ContractError::FunctionError { msg: "Static Uint function mismatch.".to_string() });
                                                }
                                                v.value = resolve_num_value_uint(
                                                    deps,
                                                    env.clone(),
                                                    nv,
                                                    &vars,
                                                )?
                                                .to_string();
                                            }
                                            UpdateFnValue::Int(nv) => {
                                                if v.kind != VariableKind::Int {
                                                    return Err(ContractError::FunctionError { msg: "Static Int function mismatch.".to_string() });
                                                }
                                                v.value = resolve_num_value_int(
                                                    deps,
                                                    env.clone(),
                                                    nv,
                                                    &vars,
                                                )?
                                                .to_string();
                                            }
                                            UpdateFnValue::Decimal(nv) => {
                                                if v.kind != VariableKind::Decimal {
                                                    return Err(ContractError::FunctionError { msg: "Static Decimal function mismatch.".to_string() });
                                                }
                                                v.value = resolve_num_value_decimal(
                                                    deps,
                                                    env.clone(),
                                                    nv,
                                                    &vars,
                                                )?
                                                .to_string();
                                            }
                                            UpdateFnValue::Timestamp(nv) => {
                                                if v.kind != VariableKind::Int {
                                                    return Err(ContractError::FunctionError { msg: "Static Timestamp function mismatch.".to_string() });
                                                }
                                                v.value = resolve_num_value_int(
                                                    deps,
                                                    env.clone(),
                                                    nv,
                                                    &vars,
                                                )?
                                                .to_string();
                                            }
                                            UpdateFnValue::BlockHeight(nv) => {
                                                if v.kind != VariableKind::Int {
                                                    return Err(ContractError::FunctionError { msg: "Static BlockHeight function mismatch.".to_string() });
                                                }
                                                v.value = resolve_num_value_int(
                                                    deps,
                                                    env.clone(),
                                                    nv,
                                                    &vars,
                                                )?
                                                .to_string();
                                            }
                                            UpdateFnValue::Bool(val) => {
                                                if v.kind != VariableKind::Bool {
                                                    return Err(ContractError::FunctionError { msg: "Static Bool function mismatch.".to_string() });
                                                }
                                                v.value = resolve_ref_bool(
                                                    deps,
                                                    env.clone(),
                                                    val,
                                                    &vars,
                                                )?
                                                .to_string();
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
                                                    return Err(ContractError::FunctionError { msg: "Static Uint function mismatch.".to_string() });
                                                }
                                                v.value = resolve_num_value_uint(
                                                    deps,
                                                    env.clone(),
                                                    nv,
                                                    &vars,
                                                )?
                                                .to_string();
                                            }
                                            UpdateFnValue::Int(nv) => {
                                                if v.kind != VariableKind::Int {
                                                    return Err(ContractError::FunctionError { msg: "Static Int function mismatch.".to_string() });
                                                }
                                                v.value = resolve_num_value_int(
                                                    deps,
                                                    env.clone(),
                                                    nv,
                                                    &vars,
                                                )?
                                                .to_string();
                                            }
                                            UpdateFnValue::Decimal(nv) => {
                                                if v.kind != VariableKind::Decimal {
                                                    return Err(ContractError::FunctionError { msg: "Static Uint function mismatch.".to_string() });
                                                }
                                                v.value = resolve_num_value_decimal(
                                                    deps,
                                                    env.clone(),
                                                    nv,
                                                    &vars,
                                                )?
                                                .to_string();
                                            }
                                            UpdateFnValue::Timestamp(nv) => {
                                                if v.kind != VariableKind::Int {
                                                    return Err(ContractError::FunctionError { msg: "Static Timestamp function mismatch.".to_string() });
                                                }
                                                v.value = resolve_num_value_int(
                                                    deps,
                                                    env.clone(),
                                                    nv,
                                                    &vars,
                                                )?
                                                .to_string();
                                            }
                                            UpdateFnValue::BlockHeight(nv) => {
                                                if v.kind != VariableKind::Int {
                                                    return Err(ContractError::FunctionError { msg: "Static BlockHeight function mismatch.".to_string() });
                                                }
                                                v.value = resolve_num_value_int(
                                                    deps,
                                                    env.clone(),
                                                    nv,
                                                    &vars,
                                                )?
                                                .to_string();
                                            }
                                            UpdateFnValue::Bool(val) => {
                                                if v.kind != VariableKind::Bool {
                                                    return Err(ContractError::FunctionError { msg: "Static Bool function mismatch.".to_string() });
                                                }
                                                v.value = resolve_ref_bool(
                                                    deps,
                                                    env.clone(),
                                                    val,
                                                    &vars,
                                                )?
                                                .to_string();
                                            }
                                        }
                                    }
                                }
                            }
                            _ => return Err(ContractError::FunctionError { msg: "Static status not supported.".to_string() }),
                        }
                    }
                }
                res.push(Variable::Static(v));
            }
            Variable::External(mut v) => {
                match v.update_fn.clone() {
                    None => (),
                    Some(update_fn) => {
                        match status {
                            JobStatus::Pending => return Err(ContractError::FunctionError { msg: "External job status pending.".to_string() }),
                            JobStatus::Executed => {
                                match update_fn.on_success {
                                    None => (),
                                    Some(on_success) => {
                                        match on_success {
                                            UpdateFnValue::Uint(nv) => {
                                                if v.kind != VariableKind::Uint {
                                                    return Err(ContractError::FunctionError { msg: "External Uint function mismatch.".to_string() });
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
                                                    return Err(ContractError::FunctionError { msg: "External Int function mismatch.".to_string() });
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
                                                if v.kind != VariableKind::Decimal {
                                                    return Err(ContractError::FunctionError { msg: "External Decimal function mismatch.".to_string() });
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
                                                    return Err(ContractError::FunctionError { msg: "External Timestamp function mismatch.".to_string() });
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
                                                    return Err(ContractError::FunctionError { msg: "External BlockHeight function mismatch.".to_string() });
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
                                                    return Err(ContractError::FunctionError { msg: "External Bool function mismatch.".to_string() });
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
                                                    return Err(ContractError::FunctionError { msg: "External Uint function mismatch.".to_string() });
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
                                                    return Err(ContractError::FunctionError { msg: "External Int function mismatch.".to_string() });
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
                                                if v.kind != VariableKind::Decimal {
                                                    return Err(ContractError::FunctionError { msg: "External Decimal function mismatch.".to_string() });
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
                                                    return Err(ContractError::FunctionError { msg: "External Timestamp function mismatch.".to_string() });
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
                                                    return Err(ContractError::FunctionError { msg: "External BlockHeight function mismatch.".to_string() });
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
                                                    return Err(ContractError::FunctionError { msg: "External Bool function mismatch.".to_string() });
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
                            _ => return Err(ContractError::FunctionError { msg: "External status not supported.".to_string() }),
                        }
                    }
                }
                res.push(Variable::External(v));
            }
            Variable::Query(mut v) => {
                match v.update_fn.clone() {
                    None => (),
                    Some(update_fn) => {
                        match status {
                            JobStatus::Pending => return Err(ContractError::FunctionError { msg: "Query job status pending.".to_string() }),
                            JobStatus::Executed => {
                                match update_fn.on_success {
                                    None => (),
                                    Some(on_success) => {
                                        match on_success {
                                            UpdateFnValue::Uint(nv) => {
                                                if v.kind != VariableKind::Uint {
                                                    return Err(ContractError::FunctionError { msg: "Query Uint function mismatch.".to_string() });
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
                                                    return Err(ContractError::FunctionError { msg: "Query Int function mismatch.".to_string() });
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
                                                if v.kind != VariableKind::Decimal {
                                                    return Err(ContractError::FunctionError { msg: "Query Decimal function mismatch.".to_string() });
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
                                                    return Err(ContractError::FunctionError { msg: "Query Timestamp function mismatch.".to_string() });
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
                                                    return Err(ContractError::FunctionError { msg: "Query Blockheighht function mismatch.".to_string() });
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
                                                    return Err(ContractError::FunctionError { msg: "Query Bool function mismatch.".to_string() });
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
                                                    return Err(ContractError::FunctionError { msg: "Query Uint function mismatch.".to_string() });
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
                                                    return Err(ContractError::FunctionError { msg: "Query Int function mismatch.".to_string() });
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
                                                if v.kind != VariableKind::Decimal {
                                                    return Err(ContractError::FunctionError { msg: "Query Decimal function mismatch.".to_string() });
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
                                                    return Err(ContractError::FunctionError { msg: "Query Timestamp function mismatch.".to_string() });
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
                                                    return Err(ContractError::FunctionError { msg: "Query BlockHeight function mismatch.".to_string() });
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
                                                    return Err(ContractError::FunctionError { msg: "Query Bool function mismatch.".to_string() });
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
                            _ => return Err(ContractError::FunctionError { msg: "Query status not supported.".to_string() }),
                        }
                    }
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
        if format!("$warp.variable.{}",n) == name {
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

pub fn string_vars_in_vector(vars: &Vec<Variable>, s: String) -> bool {
    let mut s = msg.clone();
    for var in vars {
        let name = get_var_name(var);
        s = s.replace(format!("$warp.variable.{}", name), "VAR_CHECKED")
    }
    if s.contains("$warp.variable.") {
        return false;
    }
    true
}

pub fn all_vector_vars_present(vars: &Vec<Variable>, s: String) -> bool {
    for var in vars {
        let name = get_var_name(var);
        if !s.contains(name) {
            return false;
        }
    }
    true
}

fn get_var_name(var: &Variable) -> String {
    match var.clone() {
        Variable::Static(v) => {v},
        Variable::External(v) => {v},
        Variable::Query(v) => {v}
    }.name
}

pub fn vars_valid(vars: &Vec<Variable>) -> bool {
    for var in vars {
        match var {
            Variable::Static(_) => {}
            Variable::External(v) => {
                if v.reinitialize && v.update_fn.is_some() {
                    return false;
                }
            }
            Variable::Query(v) => {
                if v.reinitialize && v.update_fn.is_some() {
                    return false;
                }
            }
        }
    }
    true
}

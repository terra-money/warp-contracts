use crate::util::path::resolve_path;
use crate::util::variable::get_var;
use crate::ContractError;
use cosmwasm_std::{
    to_vec, ContractResult, Decimal256, Deps, Env, StdError, SystemResult, Uint256,
};
use cw_storage_plus::KeyDeserialize;
use json_codec_wasm::ast::Ref;
use json_codec_wasm::Decoder;
use std::str::FromStr;
use warp_protocol::controller::condition::{
    BlockExpr, Condition, DecimalFnOp, Expr, GenExpr, IntFnOp, NumExprOp, NumExprValue, NumFnValue,
    NumOp, NumValue, StringOp, TimeExpr, TimeOp, Value,
};
use warp_protocol::controller::variable::{QueryExpr, Variable};

pub fn resolve_cond(
    deps: Deps,
    env: Env,
    cond: Condition,
    vars: &Vec<Variable>,
) -> Result<bool, ContractError> {
    match cond {
        Condition::And(conds) => {
            for cond in conds {
                if !resolve_cond(deps, env.clone(), *cond, vars)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Condition::Or(conds) => {
            for cond in conds {
                if resolve_cond(deps, env.clone(), *cond, vars)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Condition::Not(cond) => Ok(!resolve_cond(deps, env, *cond, vars)?),
        Condition::Expr(expr) => Ok(resolve_expr(deps, env, *expr, vars)?),
    }
}

pub fn resolve_expr(
    deps: Deps,
    env: Env,
    expr: Expr,
    vars: &Vec<Variable>,
) -> Result<bool, ContractError> {
    match expr {
        Expr::String(expr) => resolve_string_expr(deps, env, expr, vars),
        Expr::Uint(expr) => resolve_uint_expr(deps, env, expr, vars),
        Expr::Int(expr) => resolve_int_expr(deps, env, expr, vars),
        Expr::Decimal(expr) => resolve_decimal_expr(deps, env, expr, vars),
        Expr::Timestamp(expr) => resolve_timestamp_expr(deps, env, expr),
        Expr::BlockHeight(expr) => resolve_block_expr(deps, env, expr),
        Expr::Bool(expr) => resolve_ref_bool(deps, env, expr, vars),
    }
}

pub fn resolve_int_expr(
    deps: Deps,
    env: Env,
    expr: GenExpr<NumValue<i128, NumExprOp, IntFnOp>, NumOp>,
    vars: &Vec<Variable>,
) -> Result<bool, ContractError> {
    let left = resolve_num_value_int(deps, env.clone(), expr.left, vars)?;
    let right = resolve_num_value_int(deps, env.clone(), expr.right, vars)?;

    Ok(resolve_int_op(deps, env, left, right, expr.op))
}

pub fn resolve_num_value_int(
    deps: Deps,
    env: Env,
    value: NumValue<i128, NumExprOp, IntFnOp>,
    vars: &Vec<Variable>,
) -> Result<i128, ContractError> {
    match value {
        NumValue::Simple(value) => Ok(value),
        NumValue::Expr(expr) => resolve_num_expr_int(deps, env, expr, vars),
        NumValue::Ref(expr) => resolve_ref_int(deps, env, expr, vars),
        NumValue::Fn(expr) => resolve_num_fn_int(deps, env, expr, vars),
    }
}

fn resolve_ref_int(
    deps: Deps,
    env: Env,
    r: String,
    vars: &Vec<Variable>,
) -> Result<i128, ContractError> {
    let var = get_var(r, vars)?;
    let res = match var {
        Variable::Static(s) => str::parse::<i128>(s.value.as_str())?,
        Variable::Query(q) => resolve_query_expr_int(deps, env, q.init_fn.clone())?,
        Variable::External(_) => return Err(ContractError::Unauthorized {}), //todo: err
    };

    Ok(res)
}

fn resolve_num_fn_int(
    deps: Deps,
    env: Env,
    expr: NumFnValue<i128, NumExprOp, IntFnOp>,
    vars: &Vec<Variable>,
) -> Result<i128, ContractError> {
    let right = resolve_num_value_int(deps, env, *expr.right, vars)?;

    match expr.op {
        IntFnOp::Abs => Ok(right.abs()),
        IntFnOp::Neg => Ok(right
            .checked_mul(i128::from(-1i64))
            .ok_or(ContractError::Unauthorized {})?), //todo: err
    }
}

pub fn resolve_num_expr_int(
    deps: Deps,
    env: Env,
    expr: NumExprValue<i128, NumExprOp, IntFnOp>,
    vars: &Vec<Variable>,
) -> Result<i128, ContractError> {
    let left = resolve_num_value_int(deps, env.clone(), *expr.left, vars)?;
    let right = resolve_num_value_int(deps, env, *expr.right, vars)?;

    match expr.op {
        NumExprOp::Sub => Ok(left
            .checked_sub(right)
            .ok_or(ContractError::Unauthorized {})?), //todo: err
        NumExprOp::Add => Ok(left
            .checked_add(right)
            .ok_or(ContractError::Unauthorized {})?), //todo: err
        NumExprOp::Div => Ok(left
            .checked_div(right)
            .ok_or(ContractError::Unauthorized {})?), //todo: err
        NumExprOp::Mul => Ok(left
            .checked_mul(right)
            .ok_or(ContractError::Unauthorized {})?), //todo: err
        NumExprOp::Mod => Ok(left
            .checked_rem(right)
            .ok_or(ContractError::Unauthorized {})?), //todo: err
    }
}

pub fn resolve_uint_expr(
    deps: Deps,
    env: Env,
    expr: GenExpr<NumValue<Uint256, NumExprOp, IntFnOp>, NumOp>,
    vars: &Vec<Variable>,
) -> Result<bool, ContractError> {
    let left = resolve_num_value_uint(deps, env.clone(), expr.left, vars)?;
    let right = resolve_num_value_uint(deps, env.clone(), expr.right, vars)?;

    Ok(resolve_uint_op(deps, env, left, right, expr.op))
}

pub fn resolve_num_value_uint(
    deps: Deps,
    env: Env,
    value: NumValue<Uint256, NumExprOp, IntFnOp>,
    vars: &Vec<Variable>,
) -> Result<Uint256, ContractError> {
    match value {
        NumValue::Simple(value) => Ok(value),
        NumValue::Expr(expr) => resolve_num_expr_uint(deps, env, expr, vars),
        NumValue::Ref(expr) => resolve_ref_uint(deps, env, expr, vars), //todo: resolve ref
        NumValue::Fn(_) => Err(ContractError::Unauthorized {}),         //todo: err
    }
}

fn resolve_ref_uint(
    deps: Deps,
    env: Env,
    r: String,
    vars: &Vec<Variable>,
) -> Result<Uint256, ContractError> {
    let var = get_var(r, vars)?;
    let res = match var {
        Variable::Static(s) => Uint256::from_str(&s.value)?,
        Variable::Query(q) => resolve_query_expr_uint(deps, env, q.init_fn.clone())?,
        Variable::External(_) => return Err(ContractError::Unauthorized {}), //todo: err
    };

    Ok(res)
}

pub fn resolve_num_expr_uint(
    deps: Deps,
    env: Env,
    expr: NumExprValue<Uint256, NumExprOp, IntFnOp>,
    vars: &Vec<Variable>,
) -> Result<Uint256, ContractError> {
    let left = resolve_num_value_uint(deps, env.clone(), *expr.left, vars)?;
    let right = resolve_num_value_uint(deps, env, *expr.right, vars)?;

    match expr.op {
        NumExprOp::Sub => Ok(left
            .checked_sub(right)
            .map_err(|_| ContractError::Unauthorized {})?), //todo: err
        NumExprOp::Add => Ok(left
            .checked_add(right)
            .map_err(|_| ContractError::Unauthorized {})?), //todo: err
        NumExprOp::Div => Ok(left
            .checked_div(right)
            .map_err(|_| ContractError::Unauthorized {})?), //todo: err
        NumExprOp::Mul => Ok(left
            .checked_mul(right)
            .map_err(|_| ContractError::Unauthorized {})?), //todo: err
        NumExprOp::Mod => Ok(left
            .checked_rem(right)
            .map_err(|_| ContractError::Unauthorized {})?), //todo: err
    }
}

pub fn resolve_decimal_expr(
    deps: Deps,
    env: Env,
    expr: GenExpr<NumValue<Decimal256, NumExprOp, DecimalFnOp>, NumOp>,
    vars: &Vec<Variable>,
) -> Result<bool, ContractError> {
    let left = resolve_num_value_decimal(deps, env.clone(), expr.left, vars)?;
    let right = resolve_num_value_decimal(deps, env.clone(), expr.right, vars)?;

    Ok(resolve_decimal_op(deps, env, left, right, expr.op))
}

pub fn resolve_num_value_decimal(
    deps: Deps,
    env: Env,
    value: NumValue<Decimal256, NumExprOp, DecimalFnOp>,
    vars: &Vec<Variable>,
) -> Result<Decimal256, ContractError> {
    match value {
        NumValue::Simple(value) => Ok(value),
        NumValue::Expr(expr) => resolve_num_expr_decimal(deps, env, expr, vars),
        NumValue::Ref(expr) => resolve_ref_decimal(deps, env, expr, vars), //todo: resolve ref
        NumValue::Fn(expr) => resolve_num_fn_decimal(deps, env, expr, vars),
    }
}

fn resolve_ref_decimal(
    deps: Deps,
    env: Env,
    r: String,
    vars: &Vec<Variable>,
) -> Result<Decimal256, ContractError> {
    let var = get_var(r, vars)?;
    let res = match var {
        Variable::Static(s) => Decimal256::from_str(&s.value)?,
        Variable::Query(q) => resolve_query_expr_decimal(deps, env, q.init_fn.clone())?,
        Variable::External(_) => return Err(ContractError::Unauthorized {}), //todo: err
    };

    Ok(res)
}

fn resolve_num_fn_decimal(
    deps: Deps,
    env: Env,
    expr: NumFnValue<Decimal256, NumExprOp, DecimalFnOp>,
    vars: &Vec<Variable>,
) -> Result<Decimal256, ContractError> {
    let right = resolve_num_value_decimal(deps, env, *expr.right, vars)?;

    match expr.op {
        DecimalFnOp::Abs => Ok(right.abs_diff(Decimal256::zero())),
        DecimalFnOp::Neg => {
            Ok(right.checked_mul(Decimal256::zero().checked_sub(Decimal256::one())?)?)
        }
        DecimalFnOp::Floor => Ok(right.floor()),
        DecimalFnOp::Sqrt => Ok(right.sqrt()),
        DecimalFnOp::Ceil => Ok(right.ceil()),
    }
}

pub fn resolve_num_expr_decimal(
    deps: Deps,
    env: Env,
    expr: NumExprValue<Decimal256, NumExprOp, DecimalFnOp>,
    vars: &Vec<Variable>,
) -> Result<Decimal256, ContractError> {
    let left = resolve_num_value_decimal(deps, env.clone(), *expr.left, vars)?;
    let right = resolve_num_value_decimal(deps, env, *expr.right, vars)?;

    match expr.op {
        NumExprOp::Sub => Ok(left
            .checked_sub(right)
            .map_err(|_| ContractError::Unauthorized {})?), //todo: err
        NumExprOp::Add => Ok(left
            .checked_add(right)
            .map_err(|_| ContractError::Unauthorized {})?), //todo: err
        NumExprOp::Div => Ok(left
            .checked_div(right)
            .map_err(|_| ContractError::Unauthorized {})?), //todo: err
        NumExprOp::Mul => Ok(left
            .checked_mul(right)
            .map_err(|_| ContractError::Unauthorized {})?), //todo: err
        NumExprOp::Mod => Ok(left
            .checked_rem(right)
            .map_err(|_| ContractError::Unauthorized {})?), //todo: err
    }
}

pub fn resolve_timestamp_expr(
    _deps: Deps,
    env: Env,
    expr: TimeExpr,
) -> Result<bool, ContractError> {
    let res = match expr.op {
        TimeOp::Lt => env.block.time.seconds().lt(&expr.comparator.u64()),
        TimeOp::Gt => env.block.time.seconds().gt(&expr.comparator.u64()),
    };

    Ok(res)
}

pub fn resolve_block_expr(_deps: Deps, env: Env, expr: BlockExpr) -> Result<bool, ContractError> {
    let res = match expr.op {
        NumOp::Eq => env.block.height.eq(&expr.comparator.u64()),
        NumOp::Neq => env.block.height.ne(&expr.comparator.u64()),
        NumOp::Lt => env.block.height.lt(&expr.comparator.u64()),
        NumOp::Gt => env.block.height.gt(&expr.comparator.u64()),
        NumOp::Gte => env.block.height.ge(&expr.comparator.u64()),
        NumOp::Lte => env.block.height.le(&expr.comparator.u64()),
    };

    Ok(res)
}

pub fn resolve_uint_op(_deps: Deps, _env: Env, left: Uint256, right: Uint256, op: NumOp) -> bool {
    match op {
        NumOp::Eq => left.eq(&right),
        NumOp::Neq => left.ne(&right),
        NumOp::Lt => left.lt(&right),
        NumOp::Gt => left.gt(&right),
        NumOp::Gte => left.ge(&right),
        NumOp::Lte => left.le(&right),
    }
}

pub fn resolve_int_op(_deps: Deps, _env: Env, left: i128, right: i128, op: NumOp) -> bool {
    match op {
        NumOp::Eq => left.eq(&right),
        NumOp::Neq => left.ne(&right),
        NumOp::Lt => left.lt(&right),
        NumOp::Gt => left.gt(&right),
        NumOp::Gte => left.ge(&right),
        NumOp::Lte => left.le(&right),
    }
}

pub fn resolve_decimal_op(
    _deps: Deps,
    _env: Env,
    left: Decimal256,
    right: Decimal256,
    op: NumOp,
) -> bool {
    match op {
        NumOp::Eq => left.eq(&right),
        NumOp::Neq => left.ne(&right),
        NumOp::Lt => left.lt(&right),
        NumOp::Gt => left.gt(&right),
        NumOp::Gte => left.ge(&right),
        NumOp::Lte => left.le(&right),
    }
}

pub fn resolve_string_expr(
    deps: Deps,
    env: Env,
    expr: GenExpr<Value<String>, StringOp>,
    vars: &Vec<Variable>,
) -> Result<bool, ContractError> {
    match (expr.left, expr.right) {
        (Value::Simple(left), Value::Simple(right)) => {
            Ok(resolve_str_op(deps, env, left, right, expr.op))
        }
        (Value::Simple(left), Value::Ref(right)) => Ok(resolve_str_op(
            //todo: resolve ref
            deps,
            env.clone(),
            left,
            resolve_ref_string(deps, env, right, vars)?,
            expr.op,
        )),
        (Value::Ref(left), Value::Simple(right)) => Ok(resolve_str_op(
            //todo: resolve ref
            deps,
            env.clone(),
            resolve_ref_string(deps, env, left, vars)?,
            right,
            expr.op,
        )),
        (Value::Ref(left), Value::Ref(right)) => Ok(resolve_str_op(
            //todo: resolve ref
            deps,
            env.clone(),
            resolve_ref_string(deps, env.clone(), left, vars)?,
            resolve_ref_string(deps, env, right, vars)?,
            expr.op,
        )),
    }
}

fn resolve_ref_string(
    deps: Deps,
    env: Env,
    r: String,
    vars: &Vec<Variable>,
) -> Result<String, ContractError> {
    let var = get_var(r, vars)?;
    let res = match var {
        Variable::Static(s) => s.clone().value,
        Variable::Query(q) => resolve_query_expr_string(deps, env, q.init_fn.clone())?,
        Variable::External(_) => return Err(ContractError::Unauthorized {}), //todo: err
    };

    Ok(res)
}

pub fn resolve_str_op(_deps: Deps, _env: Env, left: String, right: String, op: StringOp) -> bool {
    match op {
        StringOp::StartsWith => left.starts_with(&right),
        StringOp::EndsWith => left.ends_with(&right),
        StringOp::Contains => left.contains(&right),
        StringOp::Eq => left.eq(&right),
        StringOp::Neq => left.ne(&right),
    }
}

pub fn resolve_query_expr(deps: Deps, _env: Env, expr: QueryExpr) -> Result<String, ContractError> {
    let raw = to_vec(&expr.query).map_err(|serialize_err| {
        StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
    })?;

    let query_result_binary = match deps.querier.raw_query(&raw) {
        SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
            "Querier system error: {}",
            system_err
        ))),
        SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(format!(
            "Querier contract error: {}",
            contract_err
        ))),
        SystemResult::Ok(ContractResult::Ok(value)) => Ok(value),
    }?;

    let query_result_str = String::from_vec(base64::decode(query_result_binary.to_string())?)?;

    Ok(query_result_str)
}

pub fn resolve_query_expr_bool(
    deps: Deps,
    env: Env,
    expr: QueryExpr,
) -> Result<bool, ContractError> {
    let query_result_str = resolve_query_expr(deps, env, expr.clone())?;
    let value = Decoder::default(query_result_str.chars()).decode()?;
    let r = Ref::new(&value);
    let resolved = resolve_path(r, expr.selector)?;

    resolved.bool().ok_or(ContractError::DecodeError {})
}

pub fn resolve_ref_bool(
    deps: Deps,
    env: Env,
    r: String,
    vars: &Vec<Variable>,
) -> Result<bool, ContractError> {
    let var = get_var(r, vars)?;
    let res = match var {
        Variable::Static(_s) => return Err(ContractError::Unauthorized {}),
        Variable::Query(q) => resolve_query_expr_bool(deps, env, q.init_fn.clone())?,
        Variable::External(_) => return Err(ContractError::Unauthorized {}), //todo: err
    };
    Ok(res)
}

pub fn resolve_query_expr_uint(
    deps: Deps,
    env: Env,
    expr: QueryExpr,
) -> Result<Uint256, ContractError> {
    let query_result_str = resolve_query_expr(deps, env, expr.clone())?;
    let value = Decoder::default(query_result_str.chars()).decode()?;
    let r = Ref::new(&value);
    let resolved = resolve_path(r, expr.selector)?;

    let str_result = Uint256::from_str(resolved.string().ok_or(ContractError::DecodeError {})?);

    let val = match str_result {
        Ok(result) => result,
        Err(_) => Uint256::from(resolved.u128().ok_or(ContractError::DecodeError {})?),
    };

    Ok(val)
}

pub fn resolve_query_expr_int(
    deps: Deps,
    env: Env,
    expr: QueryExpr,
) -> Result<i128, ContractError> {
    let query_result_str = resolve_query_expr(deps, env, expr.clone())?;
    let value = Decoder::default(query_result_str.chars()).decode()?;
    let r = Ref::new(&value);
    let resolved = resolve_path(r, expr.selector)?;

    resolved.i128().ok_or(ContractError::DecodeError {})
}

pub fn resolve_query_expr_decimal(
    deps: Deps,
    env: Env,
    expr: QueryExpr,
) -> Result<Decimal256, ContractError> {
    let query_result_str = resolve_query_expr(deps, env, expr.clone())?;
    let value = Decoder::default(query_result_str.chars()).decode()?;
    let r = Ref::new(&value);
    let resolved = resolve_path(r, expr.selector)?;

    Ok(Decimal256::from_str(
        resolved.string().ok_or(ContractError::Unauthorized {})?,
    )?)
}

pub fn resolve_query_expr_string(
    deps: Deps,
    env: Env,
    expr: QueryExpr,
) -> Result<String, ContractError> {
    let query_result_str = resolve_query_expr(deps, env, expr.clone())?;
    let value = Decoder::default(query_result_str.chars()).decode()?;
    let r = Ref::new(&value);
    let resolved = resolve_path(r, expr.selector)?;

    Ok(resolved
        .string()
        .ok_or(ContractError::DecodeError {})?
        .to_string())
}

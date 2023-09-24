use std::collections::HashMap;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal256, QueryRequest, Uint256};

use crate::condition::StringValue;

use super::condition::{DecimalFnOp, IntFnOp, NumExprOp, NumValue};

#[cw_serde]
pub enum VariableKind {
    String,
    Uint,
    Int,
    Decimal,
    Timestamp,
    Bool,
    Amount,
    Asset,
    Json,
}

#[cw_serde]
pub enum VariableValue {
    Static(String),
    Query(QueryExpr),
    External(ExternalExpr),
}

#[cw_serde]
pub struct ExternalExpr {
    pub url: String,
    pub method: Option<Method>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
    pub selector: String,
}

#[cw_serde]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[cw_serde]
pub struct QueryExpr {
    pub selector: String,
    pub query: QueryRequest<String>,
}

#[cw_serde]
pub enum ExprOp {
    Add,
    Sub,
    Div,
    Mul,
    Mod,
}

#[cw_serde]
pub enum FnOp {
    Abs,
    Neg,
    Floor,
    Sqrt,
    Ceil,
}

#[cw_serde]
pub enum FnValue {
    Uint(NumValue<Uint256, NumExprOp, IntFnOp>),
    Int(NumValue<i128, NumExprOp, IntFnOp>),
    Decimal(NumValue<Decimal256, NumExprOp, DecimalFnOp>),
    Timestamp(NumValue<i128, NumExprOp, IntFnOp>),
    BlockHeight(NumValue<i128, NumExprOp, IntFnOp>),
    Bool(String), //ref
    String(StringValue<String>),
}

#[cw_serde]
pub struct UpdateFn {
    pub on_success: Option<FnValue>,
    pub on_error: Option<FnValue>,
}

// Variable is specified as a reference value (string) in form of $warp.variable.{name}
// - variables are supplied along with the input (msg, query, template)
#[cw_serde]
pub enum Variable {
    Static(StaticVariable),
    External(ExternalVariable),
    Query(QueryVariable),
}

#[cw_serde]
pub struct StaticVariable {
    pub kind: VariableKind,
    pub name: String,
    pub encode: bool,
    pub init_fn: FnValue,
    pub reinitialize: bool,
    pub value: Option<String>, //none if uninitialized
    pub update_fn: Option<UpdateFn>,
}

#[cw_serde]
pub struct ExternalVariable {
    pub kind: VariableKind,
    pub name: String,
    pub encode: bool,
    pub init_fn: ExternalExpr,
    pub reinitialize: bool,
    pub value: Option<String>, //none if uninitialized
    pub update_fn: Option<UpdateFn>,
}

#[cw_serde]
pub struct QueryVariable {
    pub kind: VariableKind,
    pub name: String,
    pub encode: bool,
    pub init_fn: QueryExpr,
    pub reinitialize: bool,
    pub value: Option<String>, //none if uninitialized
    pub update_fn: Option<UpdateFn>,
}

use cosmwasm_schema::cw_serde;
use cosmwasm_std::QueryRequest;

use super::condition::{NumExprValue, NumFnValue};

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
}

#[cw_serde]
pub enum VariableValue {
    Static(String),
    Query(QueryExpr),
    External(ExternalExpr),
}

#[cw_serde]
pub struct ExternalExpr {
    pub name: String,
    pub url: String,
    pub method: Option<Method>,
    pub headers: Option<Vec<String>>,
    pub body: Option<String>,
    pub selector: String,
}

#[cw_serde]
pub enum Method {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE
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
pub enum UpdateFnValue {
    Expr(NumExprValue<String, ExprOp, FnOp>),
    Fn(NumFnValue<String, ExprOp, FnOp>),
}

#[cw_serde]
pub struct UpdateFn {
    pub on_success: UpdateFnValue,
    pub on_error: Option<UpdateFnValue>,
}

// Variable is specified as a reference value (string) in form of $warp.variable.{name}
// - variables are supplied along with the input (msg, query, template)
#[cw_serde]
pub enum Variable {
    Static(StaticVariable),
    External(ExternalVariable),
    Query(QueryVariable)
}

#[cw_serde]
pub struct StaticVariable {
    pub kind: VariableKind,
    pub name: String,
    pub value: Option<String>,
    pub update_fn: Option<UpdateFn>,
    pub default_value: String,
}

#[cw_serde]
pub struct ExternalVariable {
    pub kind: VariableKind,
    pub name: String,
    pub value: Option<String>,
    pub update_fn: Option<UpdateFn>,
    pub default_value: ExternalExpr,
}

#[cw_serde]
pub struct QueryVariable {
    pub kind: VariableKind,
    pub name: String,
    pub value: Option<String>,
    pub update_fn: Option<UpdateFn>,
    pub default_value: QueryExpr,
}

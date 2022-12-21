use cosmwasm_schema::cw_serde;

use super::condition::QueryExpr;

#[cw_serde]
pub enum VariableKind {
    String,
    Uint,
    Int,
    Decimal,
    Bool
}

#[cw_serde]
pub enum VariableValue {
    Static(StaticExpr),
    Query(QueryExpr),
    External(ExternalExpr)
}

#[cw_serde]
pub enum StaticVarKind {
    String,
    Uint,
    Int,
    Decimal,
    Bool,
    Amount,
    Asset,
    Timestamp,
}

#[cw_serde]
pub struct StaticExpr {
  pub kind: StaticVarKind,
  pub value: String, 
}

#[cw_serde]
pub struct ExternalExpr {
    pub url: String, 
}

// Variable is specified as a reference value (string) in form of $warp.variable.{name}
// - variables are supplied along with the input (msg, query, template)
#[cw_serde]
pub struct Variable {
    pub kind: VariableKind,
    pub name: String,
    pub default_value: VariableValue
}
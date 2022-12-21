use cosmwasm_schema::cw_serde;
use cosmwasm_std::QueryRequest;

#[cw_serde]
pub enum VariableKind {
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
pub enum VariableValue {
    Static(String),
    Query(QueryExpr),
    External(ExternalExpr),
}

#[cw_serde]
pub struct ExternalExpr {
    pub url: String,
    pub selector: String,
}

#[cw_serde]
pub struct QueryExpr {
    pub selector: String,
    pub query: QueryRequest<String>,
}

// Variable is specified as a reference value (string) in form of $warp.variable.{name}
// - variables are supplied along with the input (msg, query, template)
#[cw_serde]
pub struct Variable {
    pub kind: VariableKind,
    pub name: String,
    pub value: Option<String>,
    pub default_value: VariableValue,
}

use crate::controller::variable::Variable;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal256, Uint256, Uint64};

#[cw_serde]
pub enum Condition {
    And(Vec<Box<Condition>>),
    Or(Vec<Box<Condition>>),
    Not(Box<Condition>),
    Expr(Box<Expr>),
}

#[cw_serde]
pub struct GenExpr<Type, Op> {
    pub left: Type,
    pub op: Op,
    pub right: Type,
}

#[cw_serde]
pub struct TimeExpr {
    pub comparator: Uint64,
    pub op: TimeOp, //tmp: fix this post-comp
}

#[cw_serde]
pub struct BlockExpr {
    pub comparator: Uint64,
    pub op: NumOp,
}

#[cw_serde]
pub enum Value<T> {
    Simple(T),
    Ref(String),
}

#[cw_serde]
pub enum NumValue<T, ExprOp, FnOp> {
    Simple(T),
    Expr(NumExprValue<T, ExprOp, FnOp>),
    Ref(String),
    Fn(NumFnValue<T, ExprOp, FnOp>),
}

#[cw_serde]
pub struct NumExprValue<T, ExprOp, FnOp> {
    pub left: Box<NumValue<T, ExprOp, FnOp>>,
    pub op: ExprOp,
    pub right: Box<NumValue<T, ExprOp, FnOp>>,
}

#[cw_serde]
pub struct NumFnValue<T, ExprOp, FnOp> {
    pub op: FnOp,
    pub right: Box<NumValue<T, ExprOp, FnOp>>,
}

#[cw_serde]
pub enum NumExprOp {
    Add,
    Sub,
    Div,
    Mul,
    Mod,
}

#[cw_serde]
pub enum DecimalFnOp {
    Abs,
    Neg,
    Floor,
    Sqrt,
    Ceil,
}

#[cw_serde]
pub enum IntFnOp {
    Abs,
    Neg,
}

#[cw_serde]
pub enum Expr {
    String(GenExpr<Value<String>, StringOp>),
    Uint(GenExpr<NumValue<Uint256, NumExprOp, IntFnOp>, NumOp>),
    Int(GenExpr<NumValue<i128, NumExprOp, IntFnOp>, NumOp>),
    Decimal(GenExpr<NumValue<Decimal256, NumExprOp, DecimalFnOp>, NumOp>),
    Timestamp(TimeExpr),
    BlockHeight(BlockExpr),
    Bool(String), //ref
}

// #[cw_serde]
// pub enum BoolExpr {
//     Ref(String),
// }

#[cw_serde]
pub enum NumOp {
    Eq,
    Neq,
    Lt,
    Gt,
    Gte,
    Lte,
}

#[cw_serde]
pub enum TimeOp {
    Lt,
    Gt,
}

#[cw_serde]
pub enum StringOp {
    StartsWith,
    EndsWith,
    Contains,
    Eq,
    Neq,
}

#[cw_serde]
pub struct QueryResolveConditionMsg {
    pub condition: Condition,
    pub vars: Vec<Variable>,
}

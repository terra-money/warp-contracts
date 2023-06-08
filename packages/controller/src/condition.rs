use std::{fmt, str};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal256, Uint256, Uint64};
use json_codec_wasm::Decoder;
use json_codec_wasm::Json as CodecJson;

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
    Env(NumEnvValue),
}

#[cw_serde]
pub enum NumEnvValue {
    Time,
    BlockHeight,
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

pub struct Json {
    pub value: CodecJson,
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.value {
            CodecJson::Bool(b) => write!(f, "{}", b),
            CodecJson::I128(i) => write!(f, "{}", i),
            CodecJson::U128(u) => write!(f, "{}", u),
            CodecJson::String(s) => write!(f, "\"{}\"", s),
            CodecJson::Array(a) => {
                write!(f, "[")?;
                for (i, item) in a.iter().enumerate() {
                    if i != 0 {
                        write!(f, ",")?;
                    }
                    write!(
                        f,
                        "{}",
                        Json {
                            value: item.clone()
                        }
                    )?;
                }
                write!(f, "]")
            }
            CodecJson::Object(o) => {
                write!(f, "{{")?;
                for (i, (k, v)) in o.iter().enumerate() {
                    if i != 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "\"{}\":{}", k, Json { value: v.clone() })?;
                }
                write!(f, "}}")
            }
            CodecJson::Null => write!(f, "null"),
        }
    }
}

impl str::FromStr for Json {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = Decoder::default(s.chars())
            .decode()
            .map_err(|e| e.to_string())?;

        Ok(Json { value })
    }
}

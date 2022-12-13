use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint64};

//msg templates
#[cw_serde]
pub struct MsgTemplate {
    pub id: Uint64,
    pub owner: Addr,
    pub name: String,
    pub msg: String,
    pub formatted_str: String,
    pub vars: Vec<MsgTemplateVar>,
}

#[cw_serde]
pub struct MsgTemplateVar {
    pub name: String,
    pub path: String,
    pub ty: MsgTemplateVarType,
}

#[cw_serde]
pub enum MsgTemplateVarType {
    String,
    Uint,
    Int,
    Decimal,
    Bool,
}

#[cw_serde]
pub struct SubmitMsgTemplateMsg {
    pub name: String,
    pub msg: String,
    pub formatted_str: String,
    pub vars: Vec<MsgTemplateVar>,
}

#[cw_serde]
pub struct EditMsgTemplateMsg {
    pub name: Option<String>,
    pub msg: Option<String>,
    pub formatted_str: Option<String>,
    pub vars: Option<Vec<MsgTemplateVar>>,
}

#[cw_serde]
pub struct DeleteMsgTemplateMsg {
    pub name: Option<String>,
    pub msg: Option<String>,
    pub formatted_str: Option<String>,
    pub vars: Option<Vec<MsgTemplateVar>>,
}

#[cw_serde]
pub struct QueryMsgTemplateMsg {
    pub id: Uint64,
}

#[cw_serde]
pub struct QueryMsgTemplatesMsg {
    pub ids: Option<Vec<Uint64>>,
    pub owner: Option<Addr>,
    pub name: Option<String>,
    pub start_after: Option<Uint64>,
    pub limit: Option<u32>,
}

impl QueryMsgTemplatesMsg {
    pub fn valid_query(&self) -> bool {
        return (self.ids.is_some() as u8 + (self.owner.is_some() || self.name.is_some()) as u8)
            <= 1;
    }
}

#[cw_serde]
pub struct MsgTemplateResponse {
    pub template: MsgTemplate,
}

#[cw_serde]
pub struct MsgTemplatesResponse {
    pub templates: Vec<MsgTemplate>,
}

//query templates

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint64};

use super::variable::Variable;

//msg templates
#[cw_serde]
pub struct Template {
    pub id: Uint64,
    pub owner: Addr,
    pub name: String,
    pub vars: Vec<Variable>,
    pub kind: TemplateKind,
    pub msg: String,
    pub formatted_str: String,
}

#[cw_serde]
pub enum TemplateKind {
    Query,
    Msg,
}

#[cw_serde]
pub struct SubmitTemplateMsg {
    pub name: String,
    pub kind: TemplateKind,
    pub msg: String,
    pub formatted_str: String,
    pub vars: Vec<Variable>,
}

#[cw_serde]
pub struct EditTemplateMsg {
    pub id: Uint64,
    pub name: Option<String>,
    pub msg: Option<String>,
    pub formatted_str: Option<String>,
    pub vars: Option<Vec<Variable>>,
}

#[cw_serde]
pub struct DeleteTemplateMsg {
    pub id: Uint64,
}

#[cw_serde]
pub struct QueryTemplateMsg {
    pub id: Uint64,
}

#[cw_serde]
pub struct QueryTemplatesMsg {
    pub ids: Option<Vec<Uint64>>,
    pub owner: Option<Addr>,
    pub name: Option<String>,
    pub start_after: Option<Uint64>,
    pub limit: Option<u32>,
}

impl QueryTemplatesMsg {
    pub fn valid_query(&self) -> bool {
        return (self.ids.is_some() as u8 + (self.owner.is_some() || self.name.is_some()) as u8)
            <= 1;
    }
}

#[cw_serde]
pub struct TemplateResponse {
    pub template: Template,
}

#[cw_serde]
pub struct TemplatesResponse {
    pub templates: Vec<Template>,
}

//query templates

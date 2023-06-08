use controller::condition::Condition;
use controller::variable::Variable;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128, Uint64};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub template_fee: Uint128,
    pub fee_collector: Addr,
}

#[cw_serde]
pub struct State {
    pub current_template_id: Uint64,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub fee_collector: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    SubmitTemplate(SubmitTemplateMsg),
    EditTemplate(EditTemplateMsg),
    DeleteTemplate(DeleteTemplateMsg),
}

#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {
    #[returns(TemplateResponse)]
    QueryTemplate(QueryTemplateMsg),
    #[returns(TemplatesResponse)]
    QueryTemplates(QueryTemplatesMsg),
}

#[cw_serde]
pub struct MigrateMsg {}

//msg templates
#[cw_serde]
pub struct Template {
    pub id: Uint64,
    pub owner: Addr,
    pub name: String,
    pub vars: Vec<Variable>,
    pub msg: String,
    pub condition: Option<Condition>,
    pub formatted_str: String,
}

#[cw_serde]
pub struct SubmitTemplateMsg {
    pub name: String,
    pub msg: String,
    pub condition: Option<Condition>,
    pub formatted_str: String,
    pub vars: Vec<Variable>,
}

#[cw_serde]
pub struct EditTemplateMsg {
    pub id: Uint64,
    pub name: Option<String>,
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
        (self.ids.is_some() as u8 + (self.owner.is_some() || self.name.is_some()) as u8) <= 1
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

#[cfg(feature = "interface")]
pub use crate::{ExecuteMsg as TemplatesExecuteFns, QueryMsg as TemplatesQueryFns};
pub mod template;

use crate::template::{
    DeleteTemplateMsg, EditTemplateMsg, QueryTemplateMsg, QueryTemplatesMsg, SubmitTemplateMsg,
    Template, TemplateResponse, TemplatesResponse,
};

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128, Uint64};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub fee_denom: String,
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
    pub fee_denom: String,
    pub fee_collector: String,
    pub templates: Vec<Template>,
}

#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cw_serde]
pub enum ExecuteMsg {
    SubmitTemplate(SubmitTemplateMsg),
    EditTemplate(EditTemplateMsg),
    DeleteTemplate(DeleteTemplateMsg),

    UpdateConfig(UpdateConfigMsg),
}

#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {
    #[returns(TemplateResponse)]
    QueryTemplate(QueryTemplateMsg),
    #[returns(TemplatesResponse)]
    QueryTemplates(QueryTemplatesMsg),

    #[returns(ConfigResponse)]
    QueryConfig(QueryConfigMsg),
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct UpdateConfigMsg {
    pub owner: Option<String>,
    pub fee_denom: Option<String>,
    pub template_fee: Option<Uint128>,
    pub fee_collector: Option<String>,
}

#[cw_serde]
pub struct QueryConfigMsg {}

//responses
#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

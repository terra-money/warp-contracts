use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, QueryRequest, Uint128, Uint64};

use crate::controller::account::{
    AccountResponse, AccountsResponse, CreateAccountMsg, QueryAccountMsg, QueryAccountsMsg,
};
use crate::controller::condition::QueryResolveConditionMsg;
use crate::controller::job::{
    CreateJobMsg, DeleteJobMsg, ExecuteJobMsg, JobResponse, JobsResponse, QueryJobMsg,
    QueryJobsMsg, QueryResolveJobConditionMsg, UpdateJobMsg,
};
use crate::controller::template::{
    DeleteTemplateMsg, EditTemplateMsg, TemplateResponse, TemplatesResponse,
    QueryTemplateMsg, QueryTemplatesMsg, SubmitTemplateMsg,
};

//objects
#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub warp_account_code_id: Uint64,
    pub minimum_reward: Uint128,
    pub creation_fee_percentage: Uint128,
    pub cancellation_fee_percentage: Uint128,
}

#[cw_serde]
pub struct State {
    pub current_job_id: Uint64,
    pub current_template_id: Uint64,
}

//instantiate
#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub warp_account_code_id: Uint64,
    pub minimum_reward: Uint128,
    pub creation_fee: Uint128,
    pub cancellation_fee: Uint128,
}

//execute
#[cw_serde]
pub enum ExecuteMsg {
    CreateJob(CreateJobMsg),
    DeleteJob(DeleteJobMsg),
    UpdateJob(UpdateJobMsg),
    ExecuteJob(ExecuteJobMsg),

    CreateAccount(CreateAccountMsg),

    UpdateConfig(UpdateConfigMsg),

    SubmitTemplate(SubmitTemplateMsg),
    EditTemplate(EditTemplateMsg),
    DeleteTemplate(DeleteTemplateMsg),
}

#[cw_serde]
pub struct UpdateConfigMsg {
    pub owner: Option<String>,
    pub minimum_reward: Option<Uint128>,
    pub creation_fee_percentage: Option<Uint128>,
    pub cancellation_fee_percentage: Option<Uint128>,
}

//query
#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {
    #[returns(JobResponse)]
    QueryJob(QueryJobMsg),
    #[returns(JobsResponse)]
    QueryJobs(QueryJobsMsg),
    #[returns(bool)]
    QueryResolveJobCondition(QueryResolveJobConditionMsg),
    #[returns(bool)]
    QueryResolveCondition(QueryResolveConditionMsg),

    #[returns(SimulateResponse)]
    SimulateQuery(SimulateQueryMsg),

    #[returns(AccountResponse)]
    QueryAccount(QueryAccountMsg),
    #[returns(AccountsResponse)]
    QueryAccounts(QueryAccountsMsg),

    #[returns(ConfigResponse)]
    QueryConfig(QueryConfigMsg),

    #[returns(MsgTemplateResponse)]
    QueryTemplate(QueryTemplateMsg),
    #[returns(MsgTemplatesResponse)]
    QueryTemplates(QueryTemplatesMsg),
}

#[cw_serde]
pub struct SimulateQueryMsg {
    pub query: QueryRequest<String>,
}

#[cw_serde]
pub struct SimulateResponse {
    pub response: String,
}

#[cw_serde]
pub struct QueryConfigMsg {}

//responses
#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

//migrate
#[cw_serde]
pub struct MigrateMsg {}

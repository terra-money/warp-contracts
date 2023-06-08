use crate::account::{
    AccountResponse, AccountsResponse, CreateAccountMsg, QueryAccountMsg, QueryAccountsMsg,
};
use crate::job::{
    CreateJobMsg, DeleteJobMsg, EvictJobMsg, ExecuteJobMsg, JobResponse, JobsResponse, QueryJobMsg,
    QueryJobsMsg, UpdateJobMsg,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, QueryRequest, Uint128, Uint64};

pub mod account;
pub mod condition;
pub mod job;
pub mod variable;

//objects
#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub fee_collector: Addr,
    pub warp_account_code_id: Uint64,
    pub minimum_reward: Uint128,
    pub creation_fee_percentage: Uint64,
    pub cancellation_fee_percentage: Uint64,
    pub template_fee: Uint128,
    pub t_max: Uint64,
    pub t_min: Uint64,
    pub a_max: Uint128,
    pub a_min: Uint128,
    pub q_max: Uint64,
}

#[cw_serde]
pub struct State {
    pub current_job_id: Uint64,
    pub current_template_id: Uint64,
    pub q: Uint64,
}

//instantiate
#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub fee_collector: Option<String>,
    pub warp_account_code_id: Uint64,
    pub minimum_reward: Uint128,
    pub creation_fee: Uint64,
    pub cancellation_fee: Uint64,
    pub template_fee: Uint128,
    pub t_max: Uint64,
    pub t_min: Uint64,
    pub a_max: Uint128,
    pub a_min: Uint128,
    pub q_max: Uint64,
}

//execute
#[cw_serde]
pub enum ExecuteMsg {
    CreateJob(CreateJobMsg),
    DeleteJob(DeleteJobMsg),
    UpdateJob(UpdateJobMsg),
    ExecuteJob(ExecuteJobMsg),
    EvictJob(EvictJobMsg),

    CreateAccount(CreateAccountMsg),

    UpdateConfig(UpdateConfigMsg),
}

#[cw_serde]
pub struct UpdateConfigMsg {
    pub owner: Option<String>,
    pub fee_collector: Option<String>,
    pub minimum_reward: Option<Uint128>,
    pub creation_fee_percentage: Option<Uint64>,
    pub cancellation_fee_percentage: Option<Uint64>,
    pub template_fee: Option<Uint128>,
    pub t_max: Option<Uint64>,
    pub t_min: Option<Uint64>,
    pub a_max: Option<Uint128>,
    pub a_min: Option<Uint128>,
    pub q_max: Option<Uint64>,
}

//query
#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {
    #[returns(JobResponse)]
    QueryJob(QueryJobMsg),
    #[returns(JobsResponse)]
    QueryJobs(QueryJobsMsg),

    #[returns(SimulateResponse)]
    SimulateQuery(SimulateQueryMsg),

    #[returns(AccountResponse)]
    QueryAccount(QueryAccountMsg),
    #[returns(AccountsResponse)]
    QueryAccounts(QueryAccountsMsg),

    #[returns(ConfigResponse)]
    QueryConfig(QueryConfigMsg),
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

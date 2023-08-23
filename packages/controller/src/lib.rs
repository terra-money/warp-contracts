use crate::account::{
    AccountResponse, AccountsResponse, CreateAccountMsg, QueryAccountMsg, QueryAccountsMsg,
};
use crate::job::{
    CreateJobMsg, DeleteJobMsg, EvictJobMsg, ExecuteJobMsg, JobResponse, JobsResponse, QueryJobMsg,
    QueryJobsMsg, UpdateJobMsg,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128, Uint64};

pub mod account;
pub mod job;

//objects
#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub fee_denom: String,
    pub fee_collector: Addr,
    pub warp_account_code_id: Uint64,
    pub minimum_reward: Uint128,
    pub creation_fee_percentage: Uint64,
    pub cancellation_fee_percentage: Uint64,
    pub resolver_address: Addr,
    // maximum time for evictions
    pub t_max: Uint64,
    // minimum time for evictions
    pub t_min: Uint64,
    // maximum fee for evictions
    pub a_max: Uint128,
    // minimum fee for evictions
    pub a_min: Uint128,
    // maximum length of queue modifier for evictions
    pub q_max: Uint64,
}

#[cw_serde]
pub struct State {
    pub current_job_id: Uint64,
    // queue length
    pub q: Uint64,
}

//instantiate
#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub fee_denom: String,
    pub fee_collector: Option<String>,
    pub warp_account_code_id: Uint64,
    pub minimum_reward: Uint128,
    pub creation_fee: Uint64,
    pub cancellation_fee: Uint64,
    pub resolver_address: String,
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

    #[returns(AccountResponse)]
    QueryAccount(QueryAccountMsg),
    #[returns(AccountsResponse)]
    QueryAccounts(QueryAccountsMsg),

    #[returns(ConfigResponse)]
    QueryConfig(QueryConfigMsg),
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
pub struct MigrateMsg {
    pub warp_account_code_id: Uint64,
    pub resolver_address: String,
}

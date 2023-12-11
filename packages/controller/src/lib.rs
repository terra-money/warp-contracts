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
    // By querying job account tracker contract
    // We know all accounts owned by that user and each account's availability
    // For more detail, please refer to job account tracker contract
    pub account_tracker_address: Addr,
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
    pub creation_fee_min: Uint128,
    pub creation_fee_max: Uint128,
    pub burn_fee_min: Uint128,
    pub maintenance_fee_min: Uint128,
    pub maintenance_fee_max: Uint128,
    // duration_days fn interval [left, right]
    pub duration_days_left: Uint64,
    pub duration_days_right: Uint64,
    // queue_size fn interval [left, right]
    pub queue_size_left: Uint64,
    pub queue_size_right: Uint64,
    pub burn_fee_rate: Uint128,
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
    pub account_tracker_code_id: Uint64,
    pub minimum_reward: Uint128,
    pub creation_fee: Uint64,
    pub cancellation_fee: Uint64,
    pub resolver_address: String,
    pub t_max: Uint64,
    pub t_min: Uint64,
    pub a_max: Uint128,
    pub a_min: Uint128,
    pub q_max: Uint64,
    pub creation_fee_min: Uint128,
    pub creation_fee_max: Uint128,
    pub burn_fee_min: Uint128,
    pub maintenance_fee_min: Uint128,
    pub maintenance_fee_max: Uint128,
    // duration_days fn interval [left, right]
    pub duration_days_left: Uint64,
    pub duration_days_right: Uint64,
    // queue_size fn interval [left, right]
    pub queue_size_left: Uint64,
    pub queue_size_right: Uint64,
    pub burn_fee_rate: Uint128,
}

//execute
#[cw_serde]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    CreateJob(CreateJobMsg),
    DeleteJob(DeleteJobMsg),
    UpdateJob(UpdateJobMsg),
    ExecuteJob(ExecuteJobMsg),
    EvictJob(EvictJobMsg),

    UpdateConfig(UpdateConfigMsg),

    MigrateFreeAccounts(MigrateAccountsMsg),
    MigrateTakenAccounts(MigrateAccountsMsg),

    MigratePendingJobs(MigrateJobsMsg),
    MigrateFinishedJobs(MigrateJobsMsg),

    CreateFundingAccount(CreateFundingAccountMsg),
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
    pub creation_fee_min: Option<Uint128>,
    pub creation_fee_max: Option<Uint128>,
    pub burn_fee_min: Option<Uint128>,
    pub maintenance_fee_min: Option<Uint128>,
    pub maintenance_fee_max: Option<Uint128>,
    // duration_days fn interval [left, right]
    pub duration_days_left: Option<Uint128>,
    pub duration_days_right: Option<Uint128>,
    // queue_size fn interval [left, right]
    pub queue_size_left: Option<Uint128>,
    pub queue_size_right: Option<Uint128>,
    pub burn_fee_rate: Option<Uint128>,
}

#[cw_serde]
pub struct MigrateAccountsMsg {
    pub account_owner_addr: String,
    pub warp_account_code_id: Uint64,
    pub start_after: Option<String>,
    pub limit: u8,
}

#[cw_serde]
pub struct MigrateJobsMsg {
    pub start_after: Option<Uint64>,
    pub limit: u8,
}

#[cw_serde]
pub struct CreateFundingAccountMsg {}

//query
#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {
    #[returns(JobResponse)]
    QueryJob(QueryJobMsg),
    #[returns(JobsResponse)]
    QueryJobs(QueryJobsMsg),

    #[returns(ConfigResponse)]
    QueryConfig(QueryConfigMsg),

    #[returns(StateResponse)]
    QueryState(QueryStateMsg),
}

#[cw_serde]
pub struct QueryConfigMsg {}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cw_serde]
pub struct QueryStateMsg {}

#[cw_serde]
pub struct StateResponse {
    pub state: State,
}

#[cw_serde]
pub struct MigrateMsg {
}

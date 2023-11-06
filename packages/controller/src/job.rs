use crate::account::{AssetInfo, CwFund};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CosmosMsg, Uint128, Uint64};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

// pub enum JobFund {
//     Cw20(...),
//     Native(...),
//     Ibc(...)
// }

// 1. create_account (can potential embed funds here)
// 2. cw20_sends, native (native send or within the create_job msg itself), ibc_send (to account)
// 3. create_job msg
//      - job.funds -> withdraw_asset_from_account(...), withdraws from account to controller contract
// ...
// 4. execute_job msg
//      - job succceeded -

#[cw_serde]
pub struct Job {
    pub id: Uint64,
    // Exist if job is the follow up job of a recurring job
    pub prev_id: Option<Uint64>,
    pub owner: Addr,
    // Warp account this job is associated with, job will be executed in the context of it and pay protocol fee from it
    // As job creator can have infinite job accounts, each job account can only be used by up to 1 active job
    // So each job's fund is isolated
    pub account: Addr,
    pub last_update_time: Uint64,
    pub name: String,
    pub description: String,
    pub labels: Vec<String>,
    pub status: JobStatus,
    pub terminate_condition: Option<String>,
    pub executions: Vec<Execution>,
    pub vars: String,
    pub recurring: bool,
    pub requeue_on_evict: bool,
    pub duration_days: Uint128,
    pub reward: Uint128,
    pub assets_to_withdraw: Vec<AssetInfo>,
}

#[cw_serde]
pub enum JobVarKind {
    Query,
    External,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, Display)]
pub enum JobStatus {
    Pending,
    Executed,
    Failed,
    Cancelled,
    Evicted,
}

// Create a job using job account, if job account does not exist, create it
// Each job account will only be used for 1 job, therefore we achieve funds isolation
#[cw_serde]
pub struct Execution {
    pub condition: String,
    pub msgs: String,
}

#[cw_serde]
pub struct CreateJobMsg {
    pub name: String,
    pub description: String,
    pub labels: Vec<String>,
    pub terminate_condition: Option<String>,
    pub executions: Vec<Execution>,
    pub vars: String,
    pub recurring: bool,
    pub requeue_on_evict: bool,
    pub reward: Uint128,
    pub duration_days: Uint128,
    pub assets_to_withdraw: Option<Vec<AssetInfo>>,
    pub account_msgs: Option<Vec<CosmosMsg>>,
    pub cw_funds: Option<Vec<CwFund>>,
}

#[cw_serde]
pub struct DeleteJobMsg {
    pub id: Uint64,
}

#[cw_serde]
pub struct UpdateJobMsg {
    pub id: Uint64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub labels: Option<Vec<String>>,
    pub added_reward: Option<Uint128>,
}

#[cw_serde]
pub struct ExecuteJobMsg {
    pub id: Uint64,
    pub external_inputs: Option<Vec<ExternalInput>>,
}

#[cw_serde]
pub struct EvictJobMsg {
    pub id: Uint64,
}

#[cw_serde]
pub struct ExternalInput {
    pub name: String,
    pub input: String,
}

#[cw_serde]
pub struct QueryJobMsg {
    pub id: Uint64,
}

#[cw_serde]
pub struct QueryJobsMsg {
    pub ids: Option<Vec<Uint64>>,
    pub active: Option<bool>,
    pub owner: Option<Addr>,
    pub name: Option<String>,
    pub job_status: Option<JobStatus>,
    pub condition_status: Option<bool>,
    pub start_after: Option<JobIndex>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct JobIndex {
    pub _0: Uint128,
    pub _1: Uint64,
}

impl QueryJobsMsg {
    pub fn valid_query(&self) -> bool {
        (self.ids.is_some() as u8
            + (self.owner.is_some()
                || self.name.is_some()
                || self.job_status.is_some()
                || self.condition_status.is_some()) as u8)
            <= 1
    }
}

#[cw_serde]
pub struct QueryResolveJobConditionMsg {
    pub id: Uint64,
}

#[cw_serde]
pub struct JobResponse {
    pub job: Job,
}

#[cw_serde]
pub struct JobsResponse {
    pub jobs: Vec<Job>,
    pub total_count: usize,
}

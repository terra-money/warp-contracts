use crate::account::{AssetInfo, CwFund, WarpMsg};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Uint64};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

#[cw_serde]
pub struct Job {
    pub id: Uint64,
    // Exist if job is the follow up job of a recurring job
    pub prev_id: Option<Uint64>,
    pub owner: Addr,
    // Warp account this job is associated with, job will be executed in the context of it and
    // pay protocol fee from it. As job creator can have infinite job accounts, each job account
    // can only be used by up to 1 active job, so each job's fund is isolated
    pub account: Addr,
    // Funding account from which job fees and rewards are deducted.
    // - required for recurring jobs
    // - optionally provided for one time jobs
    pub funding_account: Option<Addr>,
    pub last_update_time: Uint64,
    pub name: String,
    pub description: String,
    pub labels: Vec<String>,
    pub status: JobStatus,
    pub terminate_condition: Option<String>,
    pub executions: Vec<Execution>,
    pub vars: String,
    pub recurring: bool,
    pub duration_days: Uint64,
    pub created_at_time: Uint64,
    pub reward: Uint128,
    // Acts like a lifecycle method - called on job termination.
    // For withdrawing assets on each job execution (recurring jobs),
    // use WithdrawAssets warp msg
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
    // exit condition for recurring jobs
    pub terminate_condition: Option<String>,
    pub executions: Vec<Execution>,
    pub vars: String,
    pub recurring: bool,
    pub reward: Uint128,
    // without funding account: operational_amount needs to equal total_fees + reward
    // with funding account: ignored, can be set to 0
    pub operational_amount: Uint128,
    pub duration_days: Uint64,
    // Acts like a lifecycle method - called on job termination.
    // For withdrawing assets on each job execution (recurring jobs),
    // use WithdrawAssets warp msg
    pub assets_to_withdraw: Option<Vec<AssetInfo>>,
    // messages that are executed via job-account when the job is created
    pub account_msgs: Option<Vec<WarpMsg>>,
    pub cw_funds: Option<Vec<CwFund>>,
    // Funding account from which job fees and rewards are deducted.
    // - required for recurring jobs
    // - optionally provided for one time jobs
    pub funding_account: Option<Addr>,
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
    pub total_count: u32,
}

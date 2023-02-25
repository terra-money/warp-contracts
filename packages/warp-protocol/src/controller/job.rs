use crate::controller::condition::Condition;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Uint64};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

use super::variable::Variable;

#[cw_serde]
pub struct Job {
    pub id: Uint64,
    pub owner: Addr,
    pub last_update_time: Uint64,
    pub name: String,
    pub status: JobStatus,
    pub condition: Condition,
    pub msgs: Vec<String>,
    pub vars: Vec<Variable>,
    pub recurring: bool,
    pub requeue_on_evict: bool,
    pub reward: Uint128,
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
pub struct CreateJobMsg {
    pub name: String,
    pub condition: Condition,
    pub msgs: Vec<String>,
    pub vars: Vec<Variable>,
    pub recurring: bool,
    pub requeue_on_evict: bool,
    pub reward: Uint128,
}

#[cw_serde]
pub struct DeleteJobMsg {
    pub id: Uint64,
}

#[cw_serde]
pub struct UpdateJobMsg {
    pub id: Uint64,
    pub name: Option<String>,
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
    // if set, will only return the specified ids.
    pub ids: Option<Vec<Uint64>>,
    pub active: Option<bool>,
    pub owner: Option<Addr>,
    pub name: Option<String>,
    pub job_status: Option<JobStatus>,
    pub condition_status: Option<bool>,
    pub start_after: Option<JobIndex>,
    pub limit: Option<u32>,
    // if set to true, it will order ascending based on the id. start_after will only use the second part of JobIndex (_1)
    pub use_id_order: Option<bool>,
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

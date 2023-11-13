pub mod condition;
pub mod variable;

use controller::{
    account::WarpMsg,
    job::{Execution, ExternalInput, JobStatus},
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::QueryRequest;
#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    ExecuteSimulateQuery(ExecuteSimulateQueryMsg),
    ExecuteValidateJobCreation(ExecuteValidateJobCreationMsg),
    ExecuteHydrateVars(ExecuteHydrateVarsMsg),
    ExecuteResolveCondition(ExecuteResolveConditionMsg),
    ExecuteApplyVarFn(ExecuteApplyVarFnMsg),
    ExecuteHydrateMsgs(ExecuteHydrateMsgsMsg),
}

#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {
    #[returns(SimulateResponse)]
    SimulateQuery(SimulateQueryMsg),
    #[returns(String)]
    QueryValidateJobCreation(QueryValidateJobCreationMsg),
    #[returns(String)]
    QueryHydrateVars(QueryHydrateVarsMsg),
    #[returns(bool)]
    QueryResolveCondition(QueryResolveConditionMsg),
    #[returns(String)]
    QueryApplyVarFn(QueryApplyVarFnMsg),
    #[returns(Vec<WarpMsg>)]
    QueryHydrateMsgs(QueryHydrateMsgsMsg),
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct ExecuteSimulateQueryMsg {
    pub query: QueryRequest<String>,
}

#[cw_serde]
pub struct ExecuteHydrateMsgsMsg {
    pub msgs: String,
    pub vars: String,
}

#[cw_serde]
pub struct ExecuteHydrateVarsMsg {
    pub vars: String,
    pub external_inputs: Option<Vec<ExternalInput>>,
    pub warp_account_addr: Option<String>,
}

#[cw_serde]
pub struct ExecuteResolveConditionMsg {
    pub condition: String,
    pub vars: String,
    pub warp_account_addr: Option<String>,
}

#[cw_serde]
pub struct ExecuteApplyVarFnMsg {
    pub vars: String,
    pub status: JobStatus,
    pub warp_account_addr: Option<String>,
}

#[cw_serde]
pub struct ExecuteValidateJobCreationMsg {
    pub terminate_condition: Option<String>,
    pub vars: String,
    pub executions: Vec<Execution>,
}

#[cw_serde]
pub struct QueryValidateJobCreationMsg {
    pub terminate_condition: Option<String>,
    pub vars: String,
    pub executions: Vec<Execution>,
}

#[cw_serde]
pub struct QueryHydrateMsgsMsg {
    pub msgs: String,
    pub vars: String,
}

#[cw_serde]
pub struct QueryHydrateVarsMsg {
    pub vars: String,
    pub external_inputs: Option<Vec<ExternalInput>>,
    pub warp_account_addr: Option<String>,
}

#[cw_serde]
pub struct QueryResolveConditionMsg {
    pub condition: String,
    pub vars: String,
    pub warp_account_addr: Option<String>,
}

#[cw_serde]
pub struct QueryApplyVarFnMsg {
    pub vars: String,
    pub status: JobStatus,
    pub warp_account_addr: Option<String>,
}

#[cw_serde]
pub struct SimulateQueryMsg {
    pub query: QueryRequest<String>,
}

#[cw_serde]
pub struct SimulateResponse {
    pub response: String,
}

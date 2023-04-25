use cosmwasm_schema::cw_serde;
use crate::controller::job::{ExternalInput, JobStatus};
use crate::resolver::condition::Condition;
use crate::resolver::variable::Variable;

pub mod condition;
pub mod variable;

#[cw_serde]
pub struct InstantiateMsg {

}

#[cw_serde]
pub enum ExecuteMsg {

}



#[cw_serde]
pub enum QueryMsg {
    ApplyVarFns(ApplyVarFnsMsg),
    ResolveCondition(ResolveConditionMsg),
    // VarsValid(VarsValidMsg),
    // HasDuplicates(HasDuplicatesMsg),
    // StringVarsInVector(StringVarsInVectorMsg),
    // AllVectorVarsPresent(AllVectorVarsPresentMsg),
    // MsgsValid(MsgsValidMsg),
    ValidateVarsAndMsgs(ValidateVarsAndMsgsMsg),
    HydrateVars(HydrateVarsMsg),
    HydrateMsgs(HydrateMsgsMsg),
}

#[cw_serde]
pub struct ApplyVarFnsMsg {
    pub vars: Vec<Variable>,
    pub status: JobStatus
}

#[cw_serde]
pub struct ApplyVarFnsResponse {
    pub vars: Vec<Variable>,
}

#[cw_serde]
pub struct VarsValidMsg {
    pub vars: Vec<Variable>
}

#[cw_serde]
pub struct HasDuplicatesMsg {
    pub vars: Vec<Variable>
}

#[cw_serde]
pub struct StringVarsInVectorMsg {
    pub vars: Vec<Variable>,
    pub s: String
}

#[cw_serde]
pub struct AllVectorVarsPresentMsg {
    pub vars: Vec<Variable>,
    pub cond_string: String,
    pub msg_string: String,
}

#[cw_serde]
pub struct MsgsValidMsg {
    pub vars: Vec<Variable>,
}

#[cw_serde]
pub struct ValidateVarsAndMsgsMsg {
    pub vars: Vec<Variable>,
    pub cond_string: String,
    pub msg_string: String,
}

#[cw_serde]
pub struct HydrateVarsMsg {
    pub vars: Vec<Variable>,
    pub external_inputs: Option<Vec<ExternalInput>>,
}

#[cw_serde]
pub struct ResolveConditionMsg {
    pub condition: Condition,
    pub vars: Vec<Variable>
}

#[cw_serde]
pub struct HydrateMsgsMsg {
    pub msgs: Vec<String>,
    pub vars: Vec<Variable>,
}


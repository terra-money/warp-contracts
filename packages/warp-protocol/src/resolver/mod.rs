use cosmwasm_schema::cw_serde;
pub mod condition;

#[cw_serde]
pub struct InstantiateMsg {

}

#[cw_serde]
pub enum ExecuteMsg {

}

#[cw_serde]
pub enum QueryMsg {
    // ResolveCondition(ResolveConditionMsg),

}
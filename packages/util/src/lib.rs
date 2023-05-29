use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, QueryRequest};

//objects
#[cw_serde]
pub struct Config {
    pub owner: Addr,
}

//instantiate
#[cw_serde]
pub struct InstantiateMsg {}

//execute
#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
pub struct UpdateConfigMsg {}

//query
#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    QueryConfig(QueryConfigMsg),
    #[returns(SimulateResponse)]
    SimulateQuery(SimulateQueryMsg),
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

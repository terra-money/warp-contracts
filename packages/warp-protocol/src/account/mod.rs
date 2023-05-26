use crate::controller::account::Fund;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CosmosMsg};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub warp_addr: Addr,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub funds: Option<Vec<Fund>>,
}

#[cw_serde]
pub struct ExecuteMsg {
    pub msgs: Vec<CosmosMsg>,
}

#[cw_serde]
pub struct ExecuteWasmMsg {}

#[cw_serde]
pub enum QueryMsg {}

#[cw_serde]
pub struct MigrateMsg {}

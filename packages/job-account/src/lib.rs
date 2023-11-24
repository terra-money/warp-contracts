use controller::account::{CwFund, WarpMsg, WarpMsgs};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin as NativeCoin, Uint64};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    // Address of warp controller contract
    pub creator_addr: Addr,
}

#[cw_serde]
pub struct InstantiateMsg {
    // User who owns this account
    pub owner: String,
    // ID of the job that is created along with the account
    pub job_id: Uint64,
    // Native funds
    pub native_funds: Vec<NativeCoin>,
    // CW20 or CW721 funds, will be transferred to account in reply of account instantiation
    pub cw_funds: Vec<CwFund>,
    // List of cosmos msgs to execute after instantiating the account
    pub msgs: Vec<WarpMsg>,
}

#[cw_serde]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    WarpMsgs(WarpMsgs),
}

#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    QueryConfig(QueryConfigMsg),
}

#[cw_serde]
pub struct QueryConfigMsg {}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cw_serde]
pub struct MigrateMsg {}

use controller::account::{AssetInfo, CwFund};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin as NativeCoin, CosmosMsg, Uint64};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    // Address of warp controller contract
    pub creator_addr: Addr,

    // Address of account tracker contract
    pub job_account_tracker_addr: Addr,
    // // Address of current warp account contract
    // pub account_addr: Addr,

    // // If occupied, occupied_by_job_id is the job id of the pending job that is using this sub account
    // pub occupied_by_job_id: Option<Uint64>,
}

#[cw_serde]
pub struct InstantiateMsg {
    // User who owns this account
    pub owner: String,
    // ID of the job that is created along with the account
    pub job_id: Uint64,

    // Account tracker tracks all accounts owned by user
    // Store it inside account for easier lookup, though most of time we only lookup account from account tracker
    // But store it enables us the other way around
    pub job_account_tracker_addr: String,

    // Only required when we are instantiate a main account
    // Since we always want to fund sub account, so we will pass this value around and send it to sub account during instantiation in create main account's reply
    pub native_funds: Vec<NativeCoin>,
    // CW20 or CW721 funds, will be transferred to account in reply of account instantiation
    pub cw_funds: Vec<CwFund>,
    // List of cosmos msgs to execute after instantiating the account
    pub msgs: Vec<CosmosMsg>,
}

#[cw_serde]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    Generic(GenericMsg),
    WithdrawAssets(WithdrawAssetsMsg),
    IbcTransfer(IbcTransferMsg),
}

#[cw_serde]
pub struct GenericMsg {
    pub msgs: Vec<CosmosMsg>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, prost::Message)]
pub struct Coin {
    #[prost(string, tag = "1")]
    pub denom: String,
    #[prost(string, tag = "2")]
    pub amount: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, prost::Message)]
pub struct TimeoutBlock {
    #[prost(uint64, optional, tag = "1")]
    pub revision_number: Option<u64>,
    #[prost(uint64, optional, tag = "2")]
    pub revision_height: Option<u64>,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, prost::Message)]
pub struct TransferMsg {
    #[prost(string, tag = "1")]
    pub source_port: String,

    #[prost(string, tag = "2")]
    pub source_channel: String,

    #[prost(message, optional, tag = "3")]
    pub token: Option<Coin>,

    #[prost(string, tag = "4")]
    pub sender: String,

    #[prost(string, tag = "5")]
    pub receiver: String,

    #[prost(message, optional, tag = "6")]
    pub timeout_block: Option<TimeoutBlock>,

    #[prost(uint64, optional, tag = "7")]
    pub timeout_timestamp: Option<u64>,

    #[prost(string, tag = "8")]
    pub memo: String,
}

#[cw_serde]
pub struct IbcTransferMsg {
    pub transfer_msg: TransferMsg,
    pub timeout_block_delta: Option<u64>,
    pub timeout_timestamp_seconds_delta: Option<u64>,
}

#[cw_serde]
pub struct WithdrawAssetsMsg {
    pub asset_infos: Vec<AssetInfo>,
}

#[cw_serde]
pub struct ExecuteWasmMsg {}

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

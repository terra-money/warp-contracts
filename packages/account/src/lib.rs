use controller::account::{AssetInfo, Fund};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, CosmosMsg, Uint64};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub warp_addr: Addr,
    pub is_sub_account: bool,
    // If current account is a main account, main_account_addr is itself,
    // If current account is a sub account, main_account_addr is its main account address
    pub main_account_addr: Addr,
}

#[cw_serde]
pub struct SubAccount {
    pub addr: String,
    // If in use, in_use_by_job_id is the job id of the job that is using this sub account
    pub in_use_by_job_id: Option<Uint64>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub msgs: Option<Vec<CosmosMsg>>,
    pub funds: Option<Vec<Fund>>,
    // By default it's false meaning it's a main account
    // If it's true, it's a sub account
    pub is_sub_account: Option<bool>,
    // Only supplied when is_sub_account is true
    // Skipped if it's instantiating a main account
    pub main_account_addr: Option<String>,
}

#[cw_serde]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    Generic(GenericMsg),
    WithdrawAssets(WithdrawAssetsMsg),
    IbcTransfer(IbcTransferMsg),
    OccupySubAccount(OccupySubAccountMsg),
    FreeSubAccount(FreeSubAccountMsg),
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

#[cw_serde]
pub struct OccupySubAccountMsg {
    pub sub_account_addr: String,
    pub job_id: Uint64,
}

#[cw_serde]
pub struct FreeSubAccountMsg {
    pub sub_account_addr: String,
}

#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    QueryConfig(QueryConfigMsg),
    #[returns(OccupiedSubAccountsResponse)]
    QueryOccupiedSubAccounts(QueryOccupiedSubAccountsMsg),
    #[returns(FreeSubAccountsResponse)]
    QueryFreeSubAccounts(QueryFreeSubAccountsMsg),
    #[returns(FirstFreeSubAccountsResponse)]
    QueryFirstFreeSubAccount(QueryFirstFreeSubAccountMsg),
    #[returns(IsSubAccountOwnedAndOccupiedResponse)]
    QueryIsSubAccountOwnedAndOccupied(QueryIsSubAccountOwnedAndOccupiedMsg),
    #[returns(IsSubAccountOwnedAndFreeResponse)]
    QueryIsSubAccountOwnedAndFree(QueryIsSubAccountOwnedAndFreeMsg),
}

#[cw_serde]
pub struct QueryConfigMsg {}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cw_serde]
pub struct QueryOccupiedSubAccountsMsg {
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct OccupiedSubAccountsResponse {
    pub sub_accounts: Vec<SubAccount>,
}

#[cw_serde]
pub struct QueryFreeSubAccountsMsg {
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct FreeSubAccountsResponse {
    pub sub_accounts: Vec<SubAccount>,
}

#[cw_serde]
pub struct QueryFirstFreeSubAccountMsg {}

#[cw_serde]
pub struct FirstFreeSubAccountsResponse {
    pub sub_account: Option<SubAccount>,
}

#[cw_serde]
pub struct QueryIsSubAccountOwnedAndOccupiedMsg {
    pub sub_account_addr: String,
}

#[cw_serde]
pub struct IsSubAccountOwnedAndOccupiedResponse {
    pub is_in_use: bool,
}

#[cw_serde]
pub struct QueryIsSubAccountOwnedAndFreeMsg {
    pub sub_account_addr: String,
}

#[cw_serde]
pub struct IsSubAccountOwnedAndFreeResponse {
    pub is_free: bool,
}

#[cw_serde]
pub struct MigrateMsg {}

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CosmosMsg, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub enum CwFund {
    Cw20(Cw20Fund),
    Cw721(Cw721Fund),
}

#[cw_serde]
pub struct Cw20Fund {
    pub contract_addr: String,
    pub amount: Uint128,
}

#[cw_serde]
pub struct Cw721Fund {
    pub contract_addr: String,
    pub token_id: String,
}

#[cw_serde]
pub enum FundTransferMsgs {
    TransferFrom(TransferFromMsg),
    TransferNft(TransferNftMsg),
}

#[cw_serde]
pub struct TransferFromMsg {
    pub owner: String,
    pub recipient: String,
    pub amount: Uint128,
}

#[cw_serde]
pub struct TransferNftMsg {
    pub recipient: String,
    pub token_id: String,
}

#[cw_serde]
pub enum Cw721ExecuteMsg {
    TransferNft { recipient: String, token_id: String },
}

#[cw_serde]
pub struct QueryLegacyAccountMsg {
    pub owner: String,
}

#[cw_serde]
pub struct QueryLegacyAccountsMsg {
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct LegacyAccount {
    pub owner: Addr,
    pub account: Addr,
}

#[cw_serde]
pub struct LegacyAccountResponse {
    pub account: LegacyAccount,
}

#[cw_serde]
pub struct LegacyAccountsResponse {
    pub accounts: Vec<LegacyAccount>,
}

#[cw_serde]
pub enum AssetInfo {
    Native(String),
    Cw20(Addr),
    Cw721(Addr, String),
}

#[cw_serde]
pub struct WarpMsgs {
    pub msgs: Vec<WarpMsg>,
}

#[cw_serde]
pub enum WarpMsg {
    Generic(CosmosMsg),
    IbcTransfer(IbcTransferMsg),
    WithdrawAssets(WithdrawAssetsMsg),
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

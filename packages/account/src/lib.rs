use controller::account::{AssetInfo, Fund};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CosmosMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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

    #[prost(message, optional, tag = "9")]
    pub fee: Option<IbcFee>
}

#[cw_serde]
pub struct IbcTransferMsg {
    pub transfer_msg: TransferMsg,
    pub timeout_block_delta: Option<u64>,
    pub timeout_timestamp_seconds_delta: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, prost::Message)]
pub struct IbcFee {
    /// **recv_fee** currently is used for compatibility with ICS-29 interface only and must be set to zero (i.e. 0untrn),
    /// because Neutron's fee module can't refund relayer for submission of Recv IBC packets due to compatibility with target chains.
    #[prost(message, repeated, tag = "1")]
    pub recv_fee: Vec<Coin>,
    /// **ack_fee** is an amount of coins to refund relayer for submitting ack message for a particular IBC packet.
    #[prost(message, repeated, tag = "2")]
    pub ack_fee: Vec<Coin>,
    /// **timeout_fee** amount of coins to refund relayer for submitting timeout message for a particular IBC packet.
    #[prost(message, repeated, tag = "3")]
    pub timeout_fee: Vec<Coin>,
}


#[cw_serde]
pub struct WithdrawAssetsMsg {
    pub asset_infos: Vec<AssetInfo>,
}

#[cw_serde]
pub struct ExecuteWasmMsg {}

#[cw_serde]
pub enum QueryMsg {
    Config,
}

#[cw_serde]
pub struct MigrateMsg {}

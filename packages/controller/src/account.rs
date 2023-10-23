use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

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
pub struct QueryMainAccountMsg {
    pub owner: String,
}

#[cw_serde]
pub struct QueryMainAccountsMsg {
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct MainAccount {
    pub owner: Addr,
    pub account: Addr,
}

#[cw_serde]
pub struct MainAccountResponse {
    pub main_account: MainAccount,
}

#[cw_serde]
pub struct MainAccountsResponse {
    pub main_accounts: Vec<MainAccount>,
}

#[cw_serde]
pub enum AssetInfo {
    Native(String),
    Cw20(Addr),
    Cw721(Addr, String),
}

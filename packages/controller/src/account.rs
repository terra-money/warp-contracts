use std::collections::{HashMap, HashSet};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct CreateAccountMsg {
    pub funds: Option<Vec<Fund>>,
}

#[cw_serde]
pub enum Fund {
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
pub struct QueryAccountMsg {
    pub owner: String,
}

#[cw_serde]
pub struct QueryAccountsMsg {
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct Account {
    pub owner: Addr,
    pub account: Addr,
}

#[cw_serde]
pub struct AccountResponse {
    pub account: Account,
}

#[cw_serde]
pub struct AccountsResponse {
    pub accounts: Vec<Account>,
}

#[cw_serde]
pub enum AssetInfo {
    Native(String),
    Cw20(Addr),
    Cw721(Addr, String),
}

// #[cw_serde]
// pub enum AssetInfoWithAmount {
//     // (native denom, amount)
//     Native(String, Uint128),
//     // (cw20 contract, amount)
//     Cw20(Addr, Uint128),
//     // amount doesn't apply to cw721
//     Cw721(Addr, String),
// }

// any downside of using hashmap vs array? hashmap makes it easier to lookup
#[cw_serde]
pub struct AssetInfoWithAmount {
    // key is denom, value is amount
    pub native: HashMap<String, Uint128>,
    // key is cw20 contract, value is amount
    pub cw20: HashMap<Addr, Uint128>,
    // key is cw721 contract, value is set of token_id
    pub cw721: HashMap<Addr, HashSet<String>>,
}

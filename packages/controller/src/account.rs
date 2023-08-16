use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Uint64};

#[cw_serde]
pub struct CreateAccountMsg {
    pub funds: Option<Vec<Fund>>,
    // If is_sub_account sets to true, we always create a new sub account, but user must have a default account first
    // If is_sub_account sets to false, we create a default account if not exist
    pub is_sub_account: Option<bool>,
    pub msgs: Option<String>,
}

#[cw_serde]
pub struct CreateAccountAndJobMsg {
    pub name: String,
    pub description: String,
    pub labels: Vec<String>,
    pub condition: String,
    pub terminate_condition: Option<String>,
    pub msgs: String,
    pub vars: String,
    pub recurring: bool,
    pub requeue_on_evict: bool,
    pub reward: Uint128,
    pub assets_to_withdraw: Option<Vec<AssetInfo>>,
    pub funds: Option<Vec<Fund>>,
    pub is_sub_account: Option<bool>,
    // optional msgs for account to run when job first created
    pub initial_msgs: Option<String>,
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
pub struct QueryAccountUsedByJobMsg {
    pub job_id: Uint64,
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
pub struct AccountUsedByJobResponse {
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

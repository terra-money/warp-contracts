use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct CreateAccountMsg {}

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

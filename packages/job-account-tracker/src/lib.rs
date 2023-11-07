use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint64};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    // Address of warp controller contract
    pub warp_addr: Addr,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub warp_addr: String,
}

#[cw_serde]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    TakeAccount(TakeAccountMsg),
    FreeAccount(FreeAccountMsg),
}

#[cw_serde]
pub struct TakeAccountMsg {
    pub account_owner_addr: String,
    pub account_addr: String,
    pub job_id: Uint64,
}

#[cw_serde]
pub struct FreeAccountMsg {
    pub account_owner_addr: String,
    pub account_addr: String,
    pub last_job_id: Uint64,
}

#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    QueryConfig(QueryConfigMsg),
    #[returns(AccountsResponse)]
    QueryTakenAccounts(QueryTakenAccountsMsg),
    #[returns(AccountsResponse)]
    QueryFreeAccounts(QueryFreeAccountsMsg),
    #[returns(FirstFreeAccountResponse)]
    QueryFirstFreeAccount(QueryFirstFreeAccountMsg),
}

#[cw_serde]
pub struct QueryConfigMsg {}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cw_serde]
pub struct QueryTakenAccountsMsg {
    pub account_owner_addr: String,
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct QueryFreeAccountsMsg {
    pub account_owner_addr: String,
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct Account {
    pub addr: Addr,
    pub taken_by_job_id: Option<Uint64>,
}

#[cw_serde]
pub struct AccountsResponse {
    pub accounts: Vec<Account>,
    pub total_count: usize,
}

#[cw_serde]
pub struct QueryFirstFreeAccountMsg {
    pub account_owner_addr: String,
}

#[cw_serde]
pub struct FirstFreeAccountResponse {
    pub account: Option<Account>,
}

#[cw_serde]
pub struct MigrateMsg {}

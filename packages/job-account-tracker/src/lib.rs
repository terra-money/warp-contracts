use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint64};

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
}

#[cw_serde]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    OccupyAccount(OccupyAccountMsg),
    FreeAccount(FreeAccountMsg),
}

#[cw_serde]
pub struct OccupyAccountMsg {
    pub account_addr: String,
    pub job_id: Uint64,
}

#[cw_serde]
pub struct FreeAccountMsg {
    pub account_addr: String,
}

#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    QueryConfig(QueryConfigMsg),
    #[returns(AccountsResponse)]
    QueryOccupiedAccounts(QueryOccupiedAccountsMsg),
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
pub struct QueryOccupiedAccountsMsg {
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct QueryFreeAccountsMsg {
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct Account {
    pub addr: Addr,
    pub occupied_by_job_id: Option<Uint64>,
}

#[cw_serde]
pub struct AccountsResponse {
    pub accounts: Vec<Account>,
    pub total_count: usize,
}

#[cw_serde]
pub struct QueryFirstFreeAccountMsg {}

#[cw_serde]
pub struct FirstFreeAccountResponse {
    pub account: Option<Account>,
}

#[cw_serde]
pub struct MigrateMsg {}

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint64};

#[cw_serde]
pub enum AccountType {
    Funding,
    Job,
}

#[cw_serde]
pub struct Account {
    pub account_type: AccountType,
    pub owner_addr: Addr,
    pub account_addr: Addr,
}

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
    TakeJobAccount(TakeJobAccountMsg),
    FreeJobAccount(FreeJobAccountMsg),
    TakeFundingAccount(TakeFundingAccountMsg),
    FreeFundingAccount(FreeFundingAccountMsg),
}

#[cw_serde]
pub struct TakeJobAccountMsg {
    pub account_owner_addr: String,
    pub account_addr: String,
    pub job_id: Uint64,
}

#[cw_serde]
pub struct FreeJobAccountMsg {
    pub account_owner_addr: String,
    pub account_addr: String,
    pub last_job_id: Uint64,
}

#[cw_serde]
pub struct TakeFundingAccountMsg {
    pub account_owner_addr: String,
    pub account_addr: String,
    pub job_id: Uint64,
}

#[cw_serde]
pub struct FreeFundingAccountMsg {
    pub account_owner_addr: String,
    pub account_addr: String,
    pub job_id: Uint64,
}

#[cw_serde]
pub struct AddFundingAccountMsg {
    pub account_owner_addr: String,
    pub account_addr: String,
}

#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    QueryConfig(QueryConfigMsg),
    #[returns(AccountsResponse)]
    QueryAccounts(QueryAccountsMsg),
    #[returns(JobAccountsResponse)]
    QueryJobAccounts(QueryJobAccountsMsg),
    #[returns(JobAccountResponse)]
    QueryJobAccount(QueryJobAccountMsg),
    #[returns(JobAccountResponse)]
    QueryFirstFreeJobAccount(QueryFirstFreeJobAccountMsg),
    #[returns(FundingAccountsResponse)]
    QueryFundingAccounts(QueryFundingAccountsMsg),
    #[returns(FundingAccountResponse)]
    QueryFundingAccount(QueryFundingAccountMsg),
    #[returns(FundingAccountResponse)]
    QueryFirstFreeFundingAccount(QueryFirstFreeFundingAccountMsg),
}

#[cw_serde]
pub struct QueryConfigMsg {}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cw_serde]
pub struct AccountsResponse {
    pub accounts: Vec<Account>,
}

#[cw_serde]
pub struct QueryAccountsMsg {
    pub account_owner_addr: String,
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub enum AccountStatus {
    Free,
    Taken,
}

#[cw_serde]
pub struct QueryJobAccountsMsg {
    pub account_owner_addr: String,
    pub account_status: AccountStatus,
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct QueryFirstFreeJobAccountMsg {
    pub account_owner_addr: String,
}

#[cw_serde]
pub struct QueryJobAccountMsg {
    pub account_owner_addr: String,
    pub account_addr: String,
}

#[cw_serde]
pub struct JobAccount {
    pub account_addr: Addr,
    pub taken_by_job_id: Uint64,
    pub account_status: AccountStatus,
}

#[cw_serde]
pub struct JobAccountsResponse {
    pub job_accounts: Vec<JobAccount>,
    pub total_count: u32,
}

#[cw_serde]
pub struct JobAccountResponse {
    pub job_account: Option<JobAccount>,
}

#[cw_serde]
pub struct QueryFundingAccountMsg {
    pub account_owner_addr: String,
    pub account_addr: String,
}

#[cw_serde]
pub struct QueryFirstFreeFundingAccountMsg {
    pub account_owner_addr: String,
}

#[cw_serde]
pub struct QueryFundingAccountsMsg {
    pub account_owner_addr: String,
    pub account_status: AccountStatus,
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct FundingAccount {
    pub account_addr: Addr,
    pub taken_by_job_ids: Vec<Uint64>,
    pub account_status: AccountStatus,
}

#[cw_serde]
pub struct FundingAccountsResponse {
    pub funding_accounts: Vec<FundingAccount>,
    pub total_count: u32,
}

#[cw_serde]
pub struct FundingAccountResponse {
    pub funding_account: Option<FundingAccount>,
}

#[cw_serde]
pub struct MigrateMsg {}

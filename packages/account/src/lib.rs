use controller::account::{AssetInfo, Fund};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, CosmosMsg, Uint64};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub warp_addr: Addr,
}

#[cw_serde]
pub struct SubAccount {
    pub addr: String,
    pub job_id: Option<Uint64>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    // if the account is created together with a job, the job_id will be passed to the account
    // but you shouldn't assume the account is always tied to the job
    // we do not force account to have 1 to 1 mapping with job
    pub job_id: Option<Uint64>,
    // cw20 / cw721 fund to deposit to the account right after init,
    // controller will parse it in the reply of account init and deposit the funds
    // native fund is passed to the account init by info.funds so it's not part of InstantiateMsg
    pub cw_funds: Vec<Fund>,
    // Stringified array of messages to execute right after init, "[]" if no messages to execute
    // It will be sent to account for executing in the reply of the init message in the controller
    pub msgs_to_execute_after_init: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Generic(GenericMsg),
    WithdrawAssets(WithdrawAssetsMsg),
    UpdateSubAccountFromFreeToInUse(UpdateSubAccountFromFreeToInUseMsg),
    UpdateSubAccountFromInUseToFree(UpdateSubAccountFromInUseToFreeMsg),
}

#[cw_serde]
pub struct GenericMsg {
    pub job_id: Option<Uint64>,
    pub msgs: Vec<CosmosMsg>,
}

#[cw_serde]
pub struct WithdrawAssetsMsg {
    pub asset_infos: Vec<AssetInfo>,
}

#[cw_serde]
pub struct ExecuteWasmMsg {}

#[cw_serde]
pub struct UpdateSubAccountFromFreeToInUseMsg {
    pub sub_account_addr: String,
    pub job_id: Uint64,
}

#[cw_serde]
pub struct UpdateSubAccountFromInUseToFreeMsg {
    pub sub_account_addr: String,
}

#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {
    #[returns(QueryConfigResponse)]
    Config,
    #[returns(QueryInUseSubAccountsResponse)]
    QueryInUseSubAccounts(QueryInUseSubAccountsMsg),
    #[returns(QueryFreeSubAccountsResponse)]
    QueryFreeSubAccounts(QueryFreeSubAccountsMsg),
    #[returns(QueryFirstFreeSubAccountsResponse)]
    QueryFirstFreeSubAccount {},
    #[returns(QueryIsSubAccountOwnedAndInUseResponse)]
    QueryIsSubAccountOwnedAndInUse(QueryIsSubAccountOwnedAndInUseMsg),
    #[returns(QueryIsSubAccountOwnedAndFreeResponse)]
    QueryIsSubAccountOwnedAndFree(QueryIsSubAccountOwnedAndFreeMsg),
}

#[cw_serde]
pub struct QueryConfigResponse {
    pub config: Config,
}

#[cw_serde]
pub struct QueryInUseSubAccountsMsg {
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct QueryInUseSubAccountsResponse {
    pub sub_accounts: Vec<SubAccount>,
}

#[cw_serde]
pub struct QueryFreeSubAccountsMsg {
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct QueryFreeSubAccountsResponse {
    pub sub_accounts: Vec<SubAccount>,
}

#[cw_serde]
pub struct QueryFirstFreeSubAccountsResponse {
    pub sub_account: SubAccount,
}

#[cw_serde]
pub struct QueryIsSubAccountOwnedAndInUseMsg {
    pub sub_account_addr: String,
}

#[cw_serde]
pub struct QueryIsSubAccountOwnedAndInUseResponse {
    pub is_in_use: bool,
}

#[cw_serde]
pub struct QueryIsSubAccountOwnedAndFreeMsg {
    pub sub_account_addr: String,
}

#[cw_serde]
pub struct QueryIsSubAccountOwnedAndFreeResponse {
    pub is_free: bool,
}

#[cw_serde]
pub struct MigrateMsg {}

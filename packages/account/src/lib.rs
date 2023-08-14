use controller::account::{AssetInfo, Fund};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{CosmosMsg, Uint64, Addr};

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
    pub funds: Option<Vec<Fund>>,
    pub job_id: Option<Uint64>,
    pub msgs: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Generic(GenericMsg),
    WithdrawAssets(WithdrawAssetsMsg),
    AddInUseSubAccount(AddInUseSubAccountMsg),
    FreeInUseSubAccount(FreeInUseSubAccountMsg),
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
pub struct AddInUseSubAccountMsg {
    pub sub_account: String,
    pub job_id: Uint64,
}

#[cw_serde]
pub struct FreeInUseSubAccountMsg {
    pub sub_account: String,
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
pub struct MigrateMsg {}

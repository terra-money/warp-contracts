use controller::account::{AssetInfo, AssetInfoWithAmount, Fund};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CosmosMsg};

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
pub enum ExecuteMsg {
    Generic(GenericMsg),
    WithdrawAssets(WithdrawAssetsMsg),
    UpdateLockedAssets(UpdateLockedAssetsMsg),
}

#[cw_serde]
pub struct GenericMsg {
    pub msgs: Vec<CosmosMsg>,
}

#[cw_serde]
pub struct WithdrawAssetsMsg {
    pub asset_infos: Vec<AssetInfo>,
}

#[cw_serde]
pub enum UpdateLockedAssetsAction {
    CreateJob,
    EvictJob,
    UpdateJobReward,
    FinishedNonRecurringJob,
    FinishedRecurringJob,
}

#[cw_serde]
pub struct UpdateLockedAssetsMsg {
    pub job_id: u64,
    // if this is a recurring job, this is the new job id
    pub new_job_id: Option<u64>,
    pub asset_info: AssetInfoWithAmount,
    pub action: UpdateLockedAssetsAction,
}

#[cw_serde]
pub struct ExecuteWasmMsg {}

#[cw_serde]
pub enum QueryMsg {}

#[cw_serde]
pub struct MigrateMsg {}

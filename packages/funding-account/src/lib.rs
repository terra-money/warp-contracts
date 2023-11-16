use controller::account::WarpMsgs;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub warp_addr: Addr,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
}

#[cw_serde]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    WarpMsgs(WarpMsgs),
}

#[cw_serde]
pub enum QueryMsg {
    Config,
}

#[cw_serde]
pub struct MigrateMsg {}

use cosmwasm_schema::cw_serde;
use cosmwasm_std::CosmosMsg::Stargate;
use cosmwasm_std::{to_binary, BankMsg, DepsMut, WasmMsg};
use cosmwasm_std::{Addr, CosmosMsg, Deps, Env, Response, StdError, StdResult, Uint128};
use cw20::{BalanceResponse, Cw20ExecuteMsg};
use cw721::{Cw721QueryMsg, OwnerOfResponse};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use prost::Message;

#[cw_serde]
pub enum CwFund {
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
pub enum AssetInfo {
    Native(String),
    Cw20(Addr),
    Cw721(Addr, String),
}

#[cw_serde]
pub struct WarpMsgs {
    pub msgs: Vec<WarpMsg>,
}

#[cw_serde]
pub enum WarpMsg {
    Generic(CosmosMsg),
    IbcTransfer(IbcTransferMsg),
    WithdrawAssets(WithdrawAssetsMsg),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, prost::Message)]
pub struct Coin {
    #[prost(string, tag = "1")]
    pub denom: String,
    #[prost(string, tag = "2")]
    pub amount: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, prost::Message)]
pub struct TimeoutBlock {
    #[prost(uint64, optional, tag = "1")]
    pub revision_number: Option<u64>,
    #[prost(uint64, optional, tag = "2")]
    pub revision_height: Option<u64>,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, prost::Message)]
pub struct TransferMsg {
    #[prost(string, tag = "1")]
    pub source_port: String,

    #[prost(string, tag = "2")]
    pub source_channel: String,

    #[prost(message, optional, tag = "3")]
    pub token: Option<Coin>,

    #[prost(string, tag = "4")]
    pub sender: String,

    #[prost(string, tag = "5")]
    pub receiver: String,

    #[prost(message, optional, tag = "6")]
    pub timeout_block: Option<TimeoutBlock>,

    #[prost(uint64, optional, tag = "7")]
    pub timeout_timestamp: Option<u64>,

    #[prost(string, tag = "8")]
    pub memo: String,
}

#[cw_serde]
pub struct IbcTransferMsg {
    pub transfer_msg: TransferMsg,
    pub timeout_block_delta: Option<u64>,
    pub timeout_timestamp_seconds_delta: Option<u64>,
}

#[cw_serde]
pub struct WithdrawAssetsMsg {
    pub asset_infos: Vec<AssetInfo>,
}

pub fn execute_warp_msgs(
    deps: DepsMut,
    env: Env,
    data: WarpMsgs,
    owner: &Addr,
) -> Result<Response, StdError> {
    let msgs = warp_msgs_to_cosmos_msgs(deps.as_ref(), env, data.msgs, owner).unwrap();

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "warp_msgs"))
}

pub fn warp_msgs_to_cosmos_msgs(
    deps: Deps,
    env: Env,
    msgs: Vec<WarpMsg>,
    owner: &Addr,
) -> Result<Vec<CosmosMsg>, StdError> {
    let result = msgs
        .into_iter()
        .flat_map(|msg| -> Vec<CosmosMsg> {
            match msg {
                WarpMsg::Generic(msg) => vec![msg],
                WarpMsg::IbcTransfer(msg) => ibc_transfer(env.clone(), msg)
                    .map(extract_messages)
                    .unwrap(),
                WarpMsg::WithdrawAssets(msg) => withdraw_assets(deps, env.clone(), msg, owner)
                    .map(extract_messages)
                    .unwrap(),
            }
        })
        .collect::<Vec<CosmosMsg>>();

    Ok(result)
}

fn extract_messages(resp: Response) -> Vec<CosmosMsg> {
    resp.messages
        .into_iter()
        .map(|cosmos_msg| cosmos_msg.msg)
        .collect()
}

pub fn withdraw_assets(
    deps: Deps,
    env: Env,
    data: WithdrawAssetsMsg,
    owner: &Addr,
) -> Result<Response, StdError> {
    let mut withdraw_msgs: Vec<CosmosMsg> = vec![];

    for asset_info in &data.asset_infos {
        match asset_info {
            AssetInfo::Native(denom) => {
                let withdraw_native_msg = withdraw_asset_native(deps, env.clone(), owner, denom)?;

                match withdraw_native_msg {
                    None => {}
                    Some(msg) => withdraw_msgs.push(msg),
                }
            }
            AssetInfo::Cw20(addr) => {
                let withdraw_cw20_msg = withdraw_asset_cw20(deps, env.clone(), owner, addr)?;

                match withdraw_cw20_msg {
                    None => {}
                    Some(msg) => withdraw_msgs.push(msg),
                }
            }
            AssetInfo::Cw721(addr, token_id) => {
                let withdraw_cw721_msg = withdraw_asset_cw721(deps, owner, addr, token_id)?;
                match withdraw_cw721_msg {
                    None => {}
                    Some(msg) => withdraw_msgs.push(msg),
                }
            }
        }
    }

    Ok(Response::new()
        .add_messages(withdraw_msgs)
        .add_attribute("action", "withdraw_assets")
        .add_attribute(
            "assets",
            serde_json_wasm::to_string(&data.asset_infos).unwrap(),
        ))
}

fn withdraw_asset_native(
    deps: Deps,
    env: Env,
    owner: &Addr,
    denom: &String,
) -> StdResult<Option<CosmosMsg>> {
    let amount = deps.querier.query_balance(env.contract.address, denom)?;

    let res = if amount.amount > Uint128::zero() {
        Some(CosmosMsg::Bank(BankMsg::Send {
            to_address: owner.to_string(),
            amount: vec![amount],
        }))
    } else {
        None
    };

    Ok(res)
}

fn withdraw_asset_cw20(
    deps: Deps,
    env: Env,
    owner: &Addr,
    token: &Addr,
) -> StdResult<Option<CosmosMsg>> {
    let amount: BalanceResponse = deps.querier.query_wasm_smart(
        token.to_string(),
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;

    let res = if amount.balance > Uint128::zero() {
        Some(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: owner.to_string(),
                amount: amount.balance,
            })?,
            funds: vec![],
        }))
    } else {
        None
    };

    Ok(res)
}

fn withdraw_asset_cw721(
    deps: Deps,
    owner: &Addr,
    token: &Addr,
    token_id: &String,
) -> StdResult<Option<CosmosMsg>> {
    let owner_query: OwnerOfResponse = deps.querier.query_wasm_smart(
        token.to_string(),
        &Cw721QueryMsg::OwnerOf {
            token_id: token_id.to_string(),
            include_expired: None,
        },
    )?;

    let res = if owner_query.owner == *owner {
        Some(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token.to_string(),
            msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: owner.to_string(),
                token_id: token_id.to_string(),
            })?,
            funds: vec![],
        }))
    } else {
        None
    };

    Ok(res)
}

pub fn ibc_transfer(env: Env, data: IbcTransferMsg) -> Result<Response, StdError> {
    let mut transfer_msg = data.transfer_msg.clone();

    if data.timeout_block_delta.is_some() && data.transfer_msg.timeout_block.is_some() {
        let block = transfer_msg.timeout_block.unwrap();
        transfer_msg.timeout_block = Some(TimeoutBlock {
            revision_number: Some(block.revision_number()),
            revision_height: Some(env.block.height + data.timeout_block_delta.unwrap()),
        })
    }

    if data.timeout_timestamp_seconds_delta.is_some() {
        transfer_msg.timeout_timestamp = Some(
            env.block
                .time
                .plus_seconds(
                    env.block.time.seconds() + data.timeout_timestamp_seconds_delta.unwrap(),
                )
                .nanos(),
        );
    }

    Ok(Response::new().add_message(Stargate {
        type_url: "/ibc.applications.transfer.v1.MsgTransfer".to_string(),
        value: transfer_msg.encode_to_vec().into(),
    }))
}

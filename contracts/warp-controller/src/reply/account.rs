use account::{FreeSubAccountMsg, GenericMsg, OccupySubAccountMsg, WithdrawAssetsMsg};
use controller::{
    account::{Account, Fund, FundTransferMsgs, TransferFromMsg, TransferNftMsg},
    job::{Job, JobStatus},
};
use cosmwasm_std::{
    to_binary, Attribute, BalanceResponse, BankMsg, BankQuery, Coin, CosmosMsg, DepsMut, Env,
    QueryRequest, Reply, Response, StdError, StdResult, SubMsgResult, Uint128, Uint64, WasmMsg,
};

use crate::{
    error::map_contract_error,
    state::{JobQueue, ACCOUNTS, CONFIG, FINISHED_JOBS, PENDING_JOBS, STATE},
    ContractError,
};

pub fn create_main_account_and_sub_account_and_job(
    mut deps: DepsMut,
    env: Env,
    msg: Reply,
) -> Result<Response, ContractError> {
    let reply = msg.result.into_result().map_err(StdError::generic_err)?;

    let event = reply
        .events
        .iter()
        .find(|event| {
            event
                .attributes
                .iter()
                .any(|attr| attr.key == "action" && attr.value == "instantiate")
        })
        .ok_or_else(|| StdError::generic_err("cannot find `instantiate` event"))?;

    let owner = event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "owner")
        .ok_or_else(|| StdError::generic_err("cannot find `owner` attribute"))?
        .value;

    let address = event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "contract_addr")
        .ok_or_else(|| StdError::generic_err("cannot find `contract_addr` attribute"))?
        .value;

    let funds: Vec<Coin> = serde_json_wasm::from_str(
        &event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "funds")
            .ok_or_else(|| StdError::generic_err("cannot find `funds` attribute"))?
            .value,
    )?;

    let cw_funds: Option<Vec<Fund>> = serde_json_wasm::from_str(
        &event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "cw_funds")
            .ok_or_else(|| StdError::generic_err("cannot find `cw_funds` attribute"))?
            .value,
    )?;

    let account_msgs: Option<Vec<CosmosMsg>> = serde_json_wasm::from_str(
        &event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "account_msgs")
            .ok_or_else(|| StdError::generic_err("cannot find `account_msgs` attribute"))?
            .value,
    )?;

    let cw_funds_vec = match cw_funds {
        None => {
            vec![]
        }
        Some(funds) => funds,
    };

    let mut msgs_vec: Vec<CosmosMsg> = vec![];

    for cw_fund in &cw_funds_vec {
        msgs_vec.push(CosmosMsg::Wasm(match cw_fund {
            Fund::Cw20(cw20_fund) => WasmMsg::Execute {
                contract_addr: deps
                    .api
                    .addr_validate(&cw20_fund.contract_addr)?
                    .to_string(),
                msg: to_binary(&FundTransferMsgs::TransferFrom(TransferFromMsg {
                    owner: owner.clone(),
                    recipient: address.clone(),
                    amount: cw20_fund.amount,
                }))?,
                funds: vec![],
            },
            Fund::Cw721(cw721_fund) => WasmMsg::Execute {
                contract_addr: deps
                    .api
                    .addr_validate(&cw721_fund.contract_addr)?
                    .to_string(),
                msg: to_binary(&FundTransferMsgs::TransferNft(TransferNftMsg {
                    recipient: address.clone(),
                    token_id: cw721_fund.token_id.clone(),
                }))?,
                funds: vec![],
            },
        }))
    }

    if let Some(msgs) = account_msgs {
        for msg in msgs {
            msgs_vec.push(msg);
        }
    }

    if ACCOUNTS().has(deps.storage, deps.api.addr_validate(&owner)?) {
        return Err(ContractError::AccountAlreadyExists {});
    }

    ACCOUNTS().save(
        deps.storage,
        deps.api.addr_validate(&owner)?,
        &Account {
            owner: deps.api.addr_validate(&owner.clone())?,
            account: deps.api.addr_validate(&address)?,
        },
    )?;
    Ok(Response::new()
        .add_attribute("action", "create_sub_account_and_job_reply")
        .add_attribute("job_id", value)
        .add_attribute("owner", owner)
        .add_attribute("account_address", address)
        .add_attribute("funds", serde_json_wasm::to_string(&funds)?)
        .add_attribute("cw_funds", serde_json_wasm::to_string(&cw_funds_vec)?)
        .add_messages(msgs_vec))
}

pub fn create_sub_account_and_job(
    mut deps: DepsMut,
    env: Env,
    msg: Reply,
) -> Result<Response, ContractError> {
    let reply = msg.result.into_result().map_err(StdError::generic_err)?;

    let event = reply
        .events
        .iter()
        .find(|event| {
            event
                .attributes
                .iter()
                .any(|attr| attr.key == "action" && attr.value == "instantiate")
        })
        .ok_or_else(|| StdError::generic_err("cannot find `instantiate` event"))?;

    let owner = event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "owner")
        .ok_or_else(|| StdError::generic_err("cannot find `owner` attribute"))?
        .value;

    let address = event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "contract_addr")
        .ok_or_else(|| StdError::generic_err("cannot find `contract_addr` attribute"))?
        .value;

    let funds: Vec<Coin> = serde_json_wasm::from_str(
        &event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "funds")
            .ok_or_else(|| StdError::generic_err("cannot find `funds` attribute"))?
            .value,
    )?;

    let cw_funds: Option<Vec<Fund>> = serde_json_wasm::from_str(
        &event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "cw_funds")
            .ok_or_else(|| StdError::generic_err("cannot find `cw_funds` attribute"))?
            .value,
    )?;

    let account_msgs: Option<Vec<CosmosMsg>> = serde_json_wasm::from_str(
        &event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "account_msgs")
            .ok_or_else(|| StdError::generic_err("cannot find `account_msgs` attribute"))?
            .value,
    )?;

    let cw_funds_vec = match cw_funds {
        None => {
            vec![]
        }
        Some(funds) => funds,
    };

    let mut msgs_vec: Vec<CosmosMsg> = vec![];

    for cw_fund in &cw_funds_vec {
        msgs_vec.push(CosmosMsg::Wasm(match cw_fund {
            Fund::Cw20(cw20_fund) => WasmMsg::Execute {
                contract_addr: deps
                    .api
                    .addr_validate(&cw20_fund.contract_addr)?
                    .to_string(),
                msg: to_binary(&FundTransferMsgs::TransferFrom(TransferFromMsg {
                    owner: owner.clone(),
                    recipient: address.clone(),
                    amount: cw20_fund.amount,
                }))?,
                funds: vec![],
            },
            Fund::Cw721(cw721_fund) => WasmMsg::Execute {
                contract_addr: deps
                    .api
                    .addr_validate(&cw721_fund.contract_addr)?
                    .to_string(),
                msg: to_binary(&FundTransferMsgs::TransferNft(TransferNftMsg {
                    recipient: address.clone(),
                    token_id: cw721_fund.token_id.clone(),
                }))?,
                funds: vec![],
            },
        }))
    }

    if let Some(msgs) = account_msgs {
        for msg in msgs {
            msgs_vec.push(msg);
        }
    }

    if ACCOUNTS().has(deps.storage, deps.api.addr_validate(&owner)?) {
        return Err(ContractError::AccountAlreadyExists {});
    }

    ACCOUNTS().save(
        deps.storage,
        deps.api.addr_validate(&owner)?,
        &Account {
            owner: deps.api.addr_validate(&owner.clone())?,
            account: deps.api.addr_validate(&address)?,
        },
    )?;
    Ok(Response::new()
        .add_attribute("action", "create_sub_account_and_job_reply")
        .add_attribute("job_id", value)
        .add_attribute("owner", owner)
        .add_attribute("account_address", address)
        .add_attribute("funds", serde_json_wasm::to_string(&funds)?)
        .add_attribute("cw_funds", serde_json_wasm::to_string(&cw_funds_vec)?)
        .add_messages(msgs_vec))
}

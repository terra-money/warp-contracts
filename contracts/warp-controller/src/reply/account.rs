use account::GenericMsg;
use controller::{
    account::{Account, Fund, FundTransferMsgs, TransferFromMsg, TransferNftMsg},
    job::Job,
};
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, Reply, Response, StdError, Uint128,
    WasmMsg,
};

use crate::{
    state::{ACCOUNTS, CONFIG, PENDING_JOBS},
    ContractError,
};

pub fn create_account(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
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
        .add_attribute("action", "save_account")
        .add_attribute("owner", owner)
        .add_attribute("account_address", address)
        .add_attribute("funds", serde_json_wasm::to_string(&funds)?)
        .add_attribute("cw_funds", serde_json_wasm::to_string(&cw_funds_vec)?)
        .add_messages(msgs_vec))
}

pub fn create_account_and_job(
    deps: DepsMut,
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

    let job_id_str = event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "job_id")
        .ok_or_else(|| StdError::generic_err("cannot find `job_id` attribute"))?
        .value;
    let job_id = u64::from_str_radix(job_id_str.as_str(), 10)?;
    let job = PENDING_JOBS().load(deps.storage, job_id)?;

    //assume reward.amount == warp token allowance
    let config = CONFIG.load(deps.storage)?;
    let fee = job.reward * Uint128::from(config.creation_fee_percentage) / Uint128::new(100);

    let reward_send_msgs = vec![
        //send reward to controller
        WasmMsg::Execute {
            contract_addr: address.clone(),
            msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {
                job_id: Some(job.id),
                msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                    to_address: env.contract.address.to_string(),
                    amount: vec![Coin::new((job.reward).u128(), config.fee_denom.clone())],
                })],
            }))?,
            funds: vec![],
        },
        WasmMsg::Execute {
            contract_addr: address.clone(),
            msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {
                job_id: Some(job.id),
                msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                    to_address: config.fee_collector.to_string(),
                    amount: vec![Coin::new((fee).u128(), config.fee_denom)],
                })],
            }))?,
            funds: vec![],
        },
    ];

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
        .add_attribute("action", "save_account")
        .add_attribute("owner", owner)
        .add_attribute("account_address", address)
        .add_attribute("funds", serde_json_wasm::to_string(&funds)?)
        .add_attribute("cw_funds", serde_json_wasm::to_string(&cw_funds_vec)?)
        .add_messages(msgs_vec)
        .add_messages(reward_send_msgs))
}

pub fn create_job_account_and_job(
    deps: DepsMut,
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

    let job_id_str = event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "job_id")
        .ok_or_else(|| StdError::generic_err("cannot find `job_id` attribute"))?
        .value;
    if job_id_str == "0" {
        return Err(ContractError::CreateAccountAndJobReplyHasInvalidJobId {});
    }
    let job_id = u64::from_str_radix(job_id_str.as_str(), 10)?;
    let job = PENDING_JOBS().load(deps.storage, job_id)?;

    PENDING_JOBS().update(deps.storage, job.id.u64(), |h| match h {
        None => Err(ContractError::JobDoesNotExist {}),
        Some(job) => Ok(Job {
            id: job.id,
            owner: job.owner,
            last_update_time: job.last_update_time,
            name: job.name,
            description: job.description,
            labels: job.labels,
            status: job.status,
            condition: job.condition,
            terminate_condition: job.terminate_condition,
            msgs: job.msgs,
            vars: job.vars,
            recurring: job.recurring,
            requeue_on_evict: job.requeue_on_evict,
            reward: job.reward,
            assets_to_withdraw: job.assets_to_withdraw,
            job_account: Some(Addr::unchecked(address.clone())),
        }),
    })?;

    //assume reward.amount == warp token allowance
    let config = CONFIG.load(deps.storage)?;
    let fee = job.reward * Uint128::from(config.creation_fee_percentage) / Uint128::new(100);

    let reward_send_msgs = vec![
        //send reward to controller
        WasmMsg::Execute {
            contract_addr: address.clone(),
            msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {
                job_id: Some(job.id),
                msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                    to_address: env.contract.address.to_string(),
                    amount: vec![Coin::new((job.reward).u128(), config.fee_denom.clone())],
                })],
            }))?,
            funds: vec![],
        },
        WasmMsg::Execute {
            contract_addr: address.clone(),
            msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {
                job_id: Some(job.id),
                msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                    to_address: config.fee_collector.to_string(),
                    amount: vec![Coin::new((fee).u128(), config.fee_denom)],
                })],
            }))?,
            funds: vec![],
        },
    ];

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

    // don't save job account, they are more like 1 time account that ties to a job or a recurring job
    Ok(Response::new()
        .add_attribute("action", "save_account")
        .add_attribute("owner", owner)
        .add_attribute("account_address", address)
        .add_attribute("funds", serde_json_wasm::to_string(&funds)?)
        .add_attribute("cw_funds", serde_json_wasm::to_string(&cw_funds_vec)?)
        .add_messages(msgs_vec)
        .add_messages(reward_send_msgs))
}

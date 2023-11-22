use cosmwasm_std::{Coin, CosmosMsg, DepsMut, Env, Reply, Response, StdError};

use controller::{
    account::{CwFund, WarpMsg},
    Config,
};

use crate::{
    state::JobQueue,
    util::msg::{
        build_account_execute_warp_msgs, build_take_funding_account_msg, build_taken_account_msg,
        build_transfer_cw20_msg, build_transfer_cw721_msg, build_transfer_native_funds_msg,
    },
    ContractError,
};

pub fn create_job_account_and_job(
    mut deps: DepsMut,
    env: Env,
    msg: Reply,
    config: Config,
) -> Result<Response, ContractError> {
    let reply = msg.result.into_result().map_err(StdError::generic_err)?;

    let job_account_event = reply
        .events
        .iter()
        .find(|event| {
            event
                .attributes
                .iter()
                .any(|attr| attr.key == "action" && attr.value == "instantiate")
        })
        .ok_or_else(|| StdError::generic_err("cannot find `instantiate` event"))?;

    let job_id_str = job_account_event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "job_id")
        .ok_or_else(|| StdError::generic_err("cannot find `job_id` attribute"))?
        .value;
    let job_id = job_id_str.as_str().parse::<u64>()?;

    let owner = job_account_event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "owner")
        .ok_or_else(|| StdError::generic_err("cannot find `owner` attribute"))?
        .value;

    let job_account_addr = deps.api.addr_validate(
        &job_account_event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "contract_addr")
            .ok_or_else(|| StdError::generic_err("cannot find `contract_addr` attribute"))?
            .value,
    )?;

    let native_funds: Vec<Coin> = serde_json_wasm::from_str(
        &job_account_event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "native_funds")
            .ok_or_else(|| StdError::generic_err("cannot find `funds` attribute"))?
            .value,
    )?;

    let cw_funds: Option<Vec<CwFund>> = serde_json_wasm::from_str(
        &job_account_event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "cw_funds")
            .ok_or_else(|| StdError::generic_err("cannot find `cw_funds` attribute"))?
            .value,
    )?;

    let account_msgs: Option<Vec<WarpMsg>> = serde_json_wasm::from_str(
        &job_account_event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "account_msgs")
            .ok_or_else(|| StdError::generic_err("cannot find `account_msgs` attribute"))?
            .value,
    )?;

    let mut job = JobQueue::get(&deps, job_id)?;
    job.account = job_account_addr.clone();
    JobQueue::sync(&mut deps, env, job.clone())?;

    let mut msgs: Vec<CosmosMsg> = vec![];

    if !native_funds.is_empty() {
        // Fund account in native coins
        msgs.push(build_transfer_native_funds_msg(
            job_account_addr.to_string(),
            native_funds.clone(),
        ))
    }

    if let Some(cw_funds) = cw_funds.clone() {
        // Fund account in CW20 / CW721 tokens
        for cw_fund in cw_funds {
            msgs.push(match cw_fund {
                CwFund::Cw20(cw20_fund) => build_transfer_cw20_msg(
                    deps.api
                        .addr_validate(&cw20_fund.contract_addr)?
                        .to_string(),
                    owner.clone(),
                    job_account_addr.clone().to_string(),
                    cw20_fund.amount,
                ),
                CwFund::Cw721(cw721_fund) => build_transfer_cw721_msg(
                    deps.api
                        .addr_validate(&cw721_fund.contract_addr)?
                        .to_string(),
                    job_account_addr.clone().to_string(),
                    cw721_fund.token_id.clone(),
                ),
            })
        }
    }

    if let Some(account_msgs) = account_msgs {
        // Account execute msgs
        msgs.push(build_account_execute_warp_msgs(
            job_account_addr.to_string(),
            account_msgs,
        ));
    }

    // Take job account
    msgs.push(build_taken_account_msg(
        config.job_account_tracker_address.to_string(),
        job.owner.to_string(),
        job_account_addr.to_string(),
        job.id,
    ));

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "create_job_account_and_job_reply")
        // .add_attribute("job_id", value)
        .add_attribute("owner", owner)
        .add_attribute("job_account_address", job_account_addr)
        .add_attribute("native_funds", serde_json_wasm::to_string(&native_funds)?)
        .add_attribute(
            "cw_funds",
            serde_json_wasm::to_string(&cw_funds.unwrap_or(vec![]))?,
        ))
}

pub fn create_funding_account_and_job(
    mut deps: DepsMut,
    env: Env,
    msg: Reply,
    config: Config,
) -> Result<Response, ContractError> {
    let reply = msg.result.into_result().map_err(StdError::generic_err)?;

    let funding_account_event = reply
        .events
        .iter()
        .find(|event| {
            event
                .attributes
                .iter()
                .any(|attr| attr.key == "action" && attr.value == "instantiate")
        })
        .ok_or_else(|| StdError::generic_err("cannot find `instantiate` event"))?;

    let job_id_str = funding_account_event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "job_id")
        .ok_or_else(|| StdError::generic_err("cannot find `job_id` attribute"))?
        .value;
    let job_id = job_id_str.as_str().parse::<u64>()?;

    let owner = funding_account_event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "owner")
        .ok_or_else(|| StdError::generic_err("cannot find `owner` attribute"))?
        .value;

    let funding_account_addr = deps.api.addr_validate(
        &funding_account_event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "contract_addr")
            .ok_or_else(|| StdError::generic_err("cannot find `contract_addr` attribute"))?
            .value,
    )?;

    let native_funds: Vec<Coin> = serde_json_wasm::from_str(
        &funding_account_event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "native_funds")
            .ok_or_else(|| StdError::generic_err("cannot find `funds` attribute"))?
            .value,
    )?;

    let mut job = JobQueue::get(&deps, job_id)?;
    job.funding_account = Some(funding_account_addr.clone());
    JobQueue::sync(&mut deps, env, job.clone())?;

    let mut msgs: Vec<CosmosMsg> = vec![];

    if !native_funds.is_empty() {
        // Fund account in native coins
        msgs.push(build_transfer_native_funds_msg(
            funding_account_addr.to_string(),
            native_funds.clone(),
        ))
    }

    // Take funding account
    msgs.push(build_take_funding_account_msg(
        config.job_account_tracker_address.to_string(),
        job.owner.to_string(),
        funding_account_addr.to_string(),
        job.id,
    ));

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "create_job_account_and_job_reply")
        // .add_attribute("job_id", value)
        .add_attribute("owner", owner)
        .add_attribute("funding_account_address", funding_account_addr)
        .add_attribute("native_funds", serde_json_wasm::to_string(&native_funds)?))
}

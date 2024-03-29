use cosmwasm_std::{Coin, CosmosMsg, DepsMut, Env, Reply, Response, StdError, Uint64};

use controller::{account::CwFund, Config};

use crate::{
    state::JobQueue,
    util::msg::{
        build_free_funding_account_msg, build_take_job_account_msg, build_transfer_cw20_msg,
        build_transfer_cw721_msg,
    },
    ContractError,
};

pub fn create_account_and_job(
    deps: DepsMut,
    env: Env,
    msg: Reply,
    config: Config,
) -> Result<Response, ContractError> {
    let reply = msg.result.into_result().map_err(StdError::generic_err)?;

    let account_event = reply
        .events
        .iter()
        .find(|event| {
            event
                .attributes
                .iter()
                .any(|attr| attr.key == "action" && attr.value == "instantiate")
        })
        .ok_or_else(|| StdError::generic_err("cannot find `instantiate` event"))?;

    let job_id_str = account_event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "job_id")
        .ok_or_else(|| StdError::generic_err("cannot find `job_id` attribute"))?
        .value;
    let job_id = job_id_str.as_str().parse::<u64>()?;

    let owner = account_event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "owner")
        .ok_or_else(|| StdError::generic_err("cannot find `owner` attribute"))?
        .value;

    let account_addr = deps.api.addr_validate(
        &account_event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "contract_addr")
            .ok_or_else(|| StdError::generic_err("cannot find `contract_addr` attribute"))?
            .value,
    )?;

    let native_funds: Vec<Coin> = serde_json_wasm::from_str(
        &account_event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "native_funds")
            .ok_or_else(|| StdError::generic_err("cannot find `native_funds` attribute"))?
            .value,
    )?;

    let cw_funds: Option<Vec<CwFund>> = serde_json_wasm::from_str(
        &account_event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "cw_funds")
            .ok_or_else(|| StdError::generic_err("cannot find `cw_funds` attribute"))?
            .value,
    )?;

    let mut job = JobQueue::get(deps.storage, job_id)?;
    job.account = account_addr.clone();
    JobQueue::sync(deps.storage, env, job.clone())?;

    let mut msgs: Vec<CosmosMsg> = vec![];

    if let Some(cw_funds) = cw_funds.clone() {
        // Fund account in CW20 / CW721 tokens
        for cw_fund in cw_funds {
            msgs.push(match cw_fund {
                CwFund::Cw20(cw20_fund) => build_transfer_cw20_msg(
                    deps.api
                        .addr_validate(&cw20_fund.contract_addr)?
                        .to_string(),
                    owner.clone(),
                    account_addr.clone().to_string(),
                    cw20_fund.amount,
                ),
                CwFund::Cw721(cw721_fund) => build_transfer_cw721_msg(
                    deps.api
                        .addr_validate(&cw721_fund.contract_addr)?
                        .to_string(),
                    account_addr.clone().to_string(),
                    cw721_fund.token_id.clone(),
                ),
            })
        }
    }

    // Take job account
    msgs.push(build_take_job_account_msg(
        config.account_tracker_address.to_string(),
        job.owner.to_string(),
        account_addr.to_string(),
        job.id,
    ));

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "create_account_and_job_reply")
        .add_attribute("job_id", job_id.to_string())
        .add_attribute("owner", owner)
        .add_attribute("account_address", account_addr)
        .add_attribute("native_funds", serde_json_wasm::to_string(&native_funds)?)
        .add_attribute(
            "cw_funds",
            serde_json_wasm::to_string(&cw_funds.unwrap_or(vec![]))?,
        ))
}

pub fn create_funding_account(
    deps: DepsMut,
    _env: Env,
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
            .ok_or_else(|| StdError::generic_err("cannot find `native_funds` attribute"))?
            .value,
    )?;

    let msgs: Vec<CosmosMsg> = vec![build_free_funding_account_msg(
        config.account_tracker_address.to_string(),
        owner.to_string(),
        funding_account_addr.to_string(),
        Uint64::from(0u64), // placeholder,
    )];

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "create_funding_account_reply")
        .add_attribute("owner", owner)
        .add_attribute("funding_account_address", funding_account_addr)
        .add_attribute("native_funds", serde_json_wasm::to_string(&native_funds)?))
}

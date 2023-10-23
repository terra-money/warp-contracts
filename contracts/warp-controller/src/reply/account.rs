use cosmwasm_std::{
    Coin, CosmosMsg, DepsMut, Env, Reply, ReplyOn, Response, StdError, SubMsg, Uint64,
};

use controller::{
    account::{CwFund, MainAccount},
    Config,
};

use crate::{
    contract::REPLY_ID_CREATE_SUB_ACCOUNT_AND_JOB,
    state::{JobQueue, ACCOUNTS},
    util::msg::{
        build_account_execute_generic_msgs, build_instantiate_warp_sub_account_msg,
        build_occupy_sub_account_msg, build_transfer_cw20_msg, build_transfer_cw721_msg,
        build_transfer_native_funds_msg,
    },
    ContractError,
};

pub fn create_main_account_and_sub_account_and_job(
    deps: DepsMut,
    env: Env,
    msg: Reply,
    config: Config,
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

    let job_id_str = event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "job_id")
        .ok_or_else(|| StdError::generic_err("cannot find `job_id` attribute"))?
        .value;
    let job_id = job_id_str.as_str().parse::<u64>()?;

    let owner = event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "owner")
        .ok_or_else(|| StdError::generic_err("cannot find `owner` attribute"))?
        .value;

    let main_account_addr = event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "contract_addr")
        .ok_or_else(|| StdError::generic_err("cannot find `contract_addr` attribute"))?
        .value;

    let native_funds: Vec<Coin> = serde_json_wasm::from_str(
        &event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "native_funds")
            .ok_or_else(|| StdError::generic_err("cannot find `funds` attribute"))?
            .value,
    )?;

    let cw_funds: Option<Vec<CwFund>> = serde_json_wasm::from_str(
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

    if ACCOUNTS().has(deps.storage, deps.api.addr_validate(&owner)?) {
        return Err(ContractError::AccountAlreadyExists {});
    }

    ACCOUNTS().save(
        deps.storage,
        deps.api.addr_validate(&owner)?,
        &MainAccount {
            owner: deps.api.addr_validate(&owner.clone())?,
            account: deps.api.addr_validate(&main_account_addr)?,
        },
    )?;

    // Create new sub account then create job in reply
    let create_sub_account_and_job_submsg = SubMsg {
        id: REPLY_ID_CREATE_SUB_ACCOUNT_AND_JOB,
        msg: build_instantiate_warp_sub_account_msg(
            Uint64::from(job_id),
            env.contract.address.to_string(),
            config.warp_account_code_id.u64(),
            owner.clone(),
            main_account_addr.clone().to_string(),
            native_funds.clone(),
            cw_funds.clone(),
            account_msgs,
        ),
        gas_limit: None,
        reply_on: ReplyOn::Always,
    };

    Ok(Response::new()
        .add_submessage(create_sub_account_and_job_submsg)
        // .add_messages(msgs)
        .add_attribute(
            "action",
            "create_main_account_and_sub_account_and_job_reply",
        )
        // .add_attribute("job_id", value)
        .add_attribute("owner", owner)
        .add_attribute("account_address", main_account_addr)
        .add_attribute("native_funds", serde_json_wasm::to_string(&native_funds)?)
        .add_attribute(
            "cw_funds",
            serde_json_wasm::to_string(&cw_funds.unwrap_or(vec![]))?,
        ))
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

    let job_id_str = event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "job_id")
        .ok_or_else(|| StdError::generic_err("cannot find `job_id` attribute"))?
        .value;
    let job_id = job_id_str.as_str().parse::<u64>()?;

    let owner = event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "owner")
        .ok_or_else(|| StdError::generic_err("cannot find `owner` attribute"))?
        .value;

    let main_account_addr = deps.api.addr_validate(
        &event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "main_account_addr")
            .ok_or_else(|| StdError::generic_err("cannot find `main_account_addr` attribute"))?
            .value,
    )?;

    let sub_account_addr = deps.api.addr_validate(
        &event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "contract_addr")
            .ok_or_else(|| StdError::generic_err("cannot find `contract_addr` attribute"))?
            .value,
    )?;

    let native_funds: Vec<Coin> = serde_json_wasm::from_str(
        &event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "native_funds")
            .ok_or_else(|| StdError::generic_err("cannot find `funds` attribute"))?
            .value,
    )?;

    let cw_funds: Option<Vec<CwFund>> = serde_json_wasm::from_str(
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

    let mut job = JobQueue::get(&deps, job_id)?;
    job.account = sub_account_addr.clone();
    JobQueue::sync(&mut deps, env, job.clone())?;

    let mut msgs: Vec<CosmosMsg> = vec![];

    if !native_funds.is_empty() {
        // Fund account in native coins
        msgs.push(build_transfer_native_funds_msg(
            sub_account_addr.clone().to_string(),
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
                    sub_account_addr.clone().to_string(),
                    cw20_fund.amount,
                ),
                CwFund::Cw721(cw721_fund) => build_transfer_cw721_msg(
                    deps.api
                        .addr_validate(&cw721_fund.contract_addr)?
                        .to_string(),
                    sub_account_addr.clone().to_string(),
                    cw721_fund.token_id.clone(),
                ),
            })
        }
    }

    if let Some(account_msgs) = account_msgs {
        // Account execute msgs
        msgs.push(build_account_execute_generic_msgs(
            sub_account_addr.to_string(),
            account_msgs,
        ));
    }

    // Occupy sub account
    msgs.push(build_occupy_sub_account_msg(
        main_account_addr.to_string(),
        sub_account_addr.to_string(),
        job.id,
    ));

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "create_sub_account_and_job_reply")
        // .add_attribute("job_id", value)
        .add_attribute("owner", owner)
        .add_attribute("account_address", sub_account_addr)
        .add_attribute("native_funds", serde_json_wasm::to_string(&native_funds)?)
        .add_attribute(
            "cw_funds",
            serde_json_wasm::to_string(&cw_funds.unwrap_or(vec![]))?,
        ))
}

use crate::state::{ACCOUNTS, CONFIG};
use crate::ContractError;
use cosmwasm_std::{
    to_binary, CosmosMsg, DepsMut, Env, MessageInfo, ReplyOn, Response, SubMsg, WasmMsg,
};
use warp_protocol::controller::account::WithdrawAssetMsg;

pub fn create_account(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let item = ACCOUNTS()
        .idx
        .account
        .item(deps.storage, info.sender.clone());

    if item?.is_some() {
        return Err(ContractError::AccountCannotCreateAccount {});
    }

    if ACCOUNTS().has(deps.storage, info.sender.clone()) {
        let account = ACCOUNTS().load(deps.storage, info.sender)?;
        return Ok(Response::new()
            .add_attribute("action", "create_account")
            .add_attribute("owner", account.owner)
            .add_attribute("account_address", account.account));
    }

    let submsg = SubMsg {
        id: 0,
        msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin: None,
            code_id: config.warp_account_code_id.u64(),
            msg: to_binary(&warp_protocol::account::InstantiateMsg {
                owner: info.sender.to_string(),
            })?,
            funds: vec![],
            label: info.sender.to_string(),
        }),
        gas_limit: None,
        reply_on: ReplyOn::Always,
    };

    Ok(Response::new()
        .add_attribute("action", "create_account")
        .add_submessage(submsg))
}

pub fn withdraw_asset(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data: WithdrawAssetMsg,
) -> Result<Response, ContractError> {
    // let q = ACCOUNTS()
    //     .idx
    //     .account
    //     .item(deps.storage, info.sender.clone())?;

    // let account = match q {
    //     None => ACCOUNTS()
    //         .load(deps.storage, info.sender)
    //         .map_err(|_e| ContractError::AccountDoesNotExist {})?,
    //     Some(q) => q.1,
    // };

    // let reward_send_msgs = vec![
    //     //send reward to controller
    //     WasmMsg::Execute {
    //         contract_addr: account.account.to_string(),
    //         msg: to_binary(&warp_protocol::account::ExecuteMsg {
    //             msgs: vec![CosmosMsg::Bank(BankMsg::Send {
    //                 to_address: env.contract.address.to_string(),
    //                 amount: vec![Coin::new((data.reward).u128(), "uluna")],
    //             })],
    //         })?,
    //         funds: vec![],
    //     },
    // ];

    // Ok(Response::new()
    //     .add_messages(reward_send_msgs)
    //     .add_attribute("action", "create_job")
    //     .add_attribute("job_id", job.id)
    //     .add_attribute("job_owner", job.owner)
    //     .add_attribute("job_name", job.name)
    //     .add_attribute("job_status", serde_json_wasm::to_string(&job.status)?)
    //     .add_attribute("job_condition", serde_json_wasm::to_string(&job.condition)?)
    //     .add_attribute("job_msgs", serde_json_wasm::to_string(&job.msgs)?)
    //     .add_attribute("job_reward", job.reward)
    //     .add_attribute("job_creation_fee", fee)
    //     .add_attribute("job_last_updated_time", job.last_update_time))
    Ok(Response::new())
}
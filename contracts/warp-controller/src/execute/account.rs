use crate::state::{ACCOUNTS, CONFIG};
use crate::ContractError;
use controller::account::CreateAccountMsg;

use cosmwasm_std::{
    to_binary, CosmosMsg, DepsMut, Env, MessageInfo, ReplyOn, Response, SubMsg, WasmMsg,
};

pub fn create_account(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data: CreateAccountMsg,
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
            msg: to_binary(&account::InstantiateMsg {
                owner: info.sender.to_string(),
                funds: data.funds,
            })?,
            funds: info.funds,
            label: info.sender.to_string(),
        }),
        gas_limit: None,
        reply_on: ReplyOn::Always,
    };

    Ok(Response::new()
        .add_attribute("action", "create_account")
        .add_submessage(submsg))
}

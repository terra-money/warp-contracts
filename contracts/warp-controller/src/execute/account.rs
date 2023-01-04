use crate::state::{ACCOUNTS, CONFIG};
use crate::ContractError;
use cosmwasm_std::{
    to_binary, CosmosMsg, DepsMut, Env, MessageInfo, ReplyOn, Response, SubMsg, WasmMsg,
};

pub fn create_account(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // sender is a warp account, warp account cannot be the owner of another warp account
    if ACCOUNTS()
        .idx
        .account
        .item(deps.storage, info.sender.clone())?
        .is_some()
    {
        return Err(ContractError::AccountCannotCreateAccount {});
    }

    // sender already owns a warp account, return the existing warp account and skip creating another one
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

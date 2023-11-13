use crate::state::CONFIG;
use crate::{execute, query, ContractError};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw_utils::nonpayable;
use job_account::{Config, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let instantiated_account_addr = env.contract.address;

    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&msg.owner)?,
            creator_addr: info.sender,
        },
    )?;

    Ok(Response::new()
        .add_messages(msg.msgs.clone())
        .add_attribute("action", "instantiate")
        .add_attribute("job_id", msg.job_id)
        .add_attribute("contract_addr", instantiated_account_addr)
        .add_attribute("owner", msg.owner)
        .add_attribute(
            "native_funds",
            serde_json_wasm::to_string(&msg.native_funds)?,
        )
        .add_attribute("cw_funds", serde_json_wasm::to_string(&msg.cw_funds)?)
        .add_attribute("account_msgs", serde_json_wasm::to_string(&msg.msgs)?))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner && info.sender != config.creator_addr {
        return Err(ContractError::Unauthorized {});
    }
    match msg {
        ExecuteMsg::Generic(data) => Ok(Response::new()
            .add_messages(data.msgs)
            .add_attribute("action", "generic")),
        ExecuteMsg::WithdrawAssets(data) => {
            nonpayable(&info).unwrap();
            execute::withdraw::withdraw_assets(deps.as_ref(), env, data, config)
        }
        ExecuteMsg::IbcTransfer(data) => execute::ibc::ibc_transfer(env, data),
        ExecuteMsg::WarpMsgs(data) => execute::msgs::execute_warp_msgs(deps, env, data, config),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryConfig(_) => to_binary(&query::account::query_config(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
}

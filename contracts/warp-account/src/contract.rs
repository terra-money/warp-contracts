use crate::state::CONFIG;
use crate::{execute, query, ContractError};
use account::{Config, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let instantiated_account_addr = env.contract.address;
    let main_account_addr = if msg.is_sub_account.unwrap_or(false) {
        deps.api.addr_validate(&msg.main_account_addr.unwrap())?
    } else {
        instantiated_account_addr.clone()
    };

    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&msg.owner)?,
            warp_addr: info.sender,
            is_sub_account: msg.is_sub_account.unwrap_or(false),
            main_account_addr: main_account_addr.clone(),
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_addr", instantiated_account_addr)
        .add_attribute(
            "is_sub_account",
            format!("{}", msg.is_sub_account.unwrap_or(false)),
        )
        .add_attribute("main_account_addr", main_account_addr)
        .add_attribute("owner", msg.owner)
        .add_attribute("funds", serde_json_wasm::to_string(&info.funds)?)
        .add_attribute("cw_funds", serde_json_wasm::to_string(&msg.funds)?)
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
    if info.sender != config.owner && info.sender != config.warp_addr {
        return Err(ContractError::Unauthorized {});
    }
    match msg {
        ExecuteMsg::Generic(data) => Ok(Response::new()
            .add_messages(data.msgs)
            .add_attribute("action", "generic")),
        ExecuteMsg::WithdrawAssets(data) => {
            execute::withdraw::withdraw_assets(deps, env, info, data)
        }
        ExecuteMsg::IbcTransfer(data) => execute::ibc::ibc_transfer(env, data),
        ExecuteMsg::OccupySubAccount(data) => execute::account::occupy_sub_account(deps, env, data),
        ExecuteMsg::FreeSubAccount(data) => execute::account::free_sub_account(deps, env, data),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryConfig(_) => to_binary(&query::account::query_config(deps)?),
        QueryMsg::QueryOccupiedSubAccounts(data) => {
            to_binary(&query::account::query_occupied_sub_accounts(deps, data)?)
        }
        QueryMsg::QueryFreeSubAccounts(data) => {
            to_binary(&query::account::query_free_sub_accounts(deps, data)?)
        }
        QueryMsg::QueryFirstFreeSubAccount(_) => {
            to_binary(&query::account::query_first_free_sub_account(deps)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
}

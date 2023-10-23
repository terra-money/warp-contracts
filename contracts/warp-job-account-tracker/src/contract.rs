use crate::state::CONFIG;
use crate::{execute, query, ContractError};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw_utils::nonpayable;
use job_account_tracker::{Config, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

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
        .add_attribute("action", "instantiate")
        .add_attribute("contract_addr", instantiated_account_addr.clone())
        .add_attribute("owner", msg.owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner && info.sender != config.creator_addr {
        return Err(ContractError::Unauthorized {});
    }
    match msg {
        ExecuteMsg::OccupyAccount(data) => {
            nonpayable(&info).unwrap();
            execute::account::occupy_account(deps, data)
        }
        ExecuteMsg::FreeAccount(data) => {
            nonpayable(&info).unwrap();
            execute::account::free_account(deps, data)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryConfig(_) => to_binary(&query::account::query_config(deps)?),
        QueryMsg::QueryOccupiedAccounts(data) => {
            to_binary(&query::account::query_occupied_accounts(deps, data)?)
        }
        QueryMsg::QueryFreeAccounts(data) => {
            to_binary(&query::account::query_free_accounts(deps, data)?)
        }
        QueryMsg::QueryFirstFreeAccount(_) => {
            to_binary(&query::account::query_first_free_account(deps)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
}

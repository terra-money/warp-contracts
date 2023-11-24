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
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let instantiated_account_addr = env.contract.address;

    CONFIG.save(
        deps.storage,
        &Config {
            // owner: deps.api.addr_validate(&msg.owner)?,
            // creator_addr: info.sender,
            admin: deps.api.addr_validate(&msg.admin)?,
            warp_addr: deps.api.addr_validate(&msg.warp_addr)?,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_addr", instantiated_account_addr.clone())
        .add_attribute("job_account_tracker", instantiated_account_addr)
        .add_attribute("admin", msg.admin)
        .add_attribute("warp_addr", msg.warp_addr))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin && info.sender != config.warp_addr {
        return Err(ContractError::Unauthorized {});
    }

    match msg {
        ExecuteMsg::TakeAccount(data) => {
            nonpayable(&info).unwrap();
            execute::account::taken_account(deps, data)
        }
        ExecuteMsg::FreeAccount(data) => {
            nonpayable(&info).unwrap();
            execute::account::free_account(deps, data)
        }
        ExecuteMsg::TakeFundingAccount(data) => {
            nonpayable(&info).unwrap();
            execute::account::take_funding_account(deps, data)
        }
        ExecuteMsg::FreeFundingAccount(data) => {
            nonpayable(&info).unwrap();
            execute::account::free_funding_account(deps, data)
        }
        ExecuteMsg::AddFundingAccount(data) => {
            nonpayable(&info).unwrap();
            execute::account::add_funding_account(deps, data)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryConfig(_) => to_binary(&query::account::query_config(deps)?),
        QueryMsg::QueryTakenAccounts(data) => {
            to_binary(&query::account::query_taken_accounts(deps, data)?)
        }
        QueryMsg::QueryFreeAccounts(data) => {
            to_binary(&query::account::query_free_accounts(deps, data)?)
        }
        QueryMsg::QueryFirstFreeAccount(data) => {
            to_binary(&query::account::query_first_free_account(deps, data)?)
        }
        QueryMsg::QueryFreeAccount(data) => {
            to_binary(&query::account::query_free_account(deps, data)?)
        }
        QueryMsg::QueryFundingAccounts(data) => {
            to_binary(&query::account::query_funding_accounts(deps, data)?)
        }
        QueryMsg::QueryFundingAccount(data) => {
            to_binary(&query::account::query_funding_account(deps, data)?)
        }
        QueryMsg::QueryFirstFreeFundingAccount(data) => to_binary(
            &query::account::query_first_free_funding_account(deps, data)?,
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
}

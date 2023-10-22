use crate::state::CONFIG;
use crate::{execute, query, ContractError};
use account::{Config, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SubAccountConfig};
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

    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&msg.owner)?,
            creator_addr: info.sender,
            account_addr: instantiated_account_addr.clone(),
            sub_account_config: if msg.is_sub_account {
                Some(SubAccountConfig {
                    main_account_addr: deps
                        .api
                        .addr_validate(&msg.main_account_addr.clone().unwrap())?,
                    occupied_by_job_id: None,
                })
            } else {
                None
            },
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_addr", instantiated_account_addr.clone())
        .add_attribute("is_sub_account", format!("{}", msg.is_sub_account))
        .add_attribute(
            "main_account_addr",
            msg.main_account_addr
                .unwrap_or(instantiated_account_addr.to_string()),
        )
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
    if info.sender != config.owner && info.sender != config.creator_addr {
        return Err(ContractError::Unauthorized {});
    }
    match msg {
        ExecuteMsg::Generic(data) => Ok(Response::new()
            .add_messages(data.msgs)
            .add_attribute("action", "generic")),
        ExecuteMsg::WithdrawAssets(data) => {
            execute::withdraw::withdraw_assets(deps, env, data, config)
        }
        ExecuteMsg::IbcTransfer(data) => execute::ibc::ibc_transfer(env, data),
        ExecuteMsg::OccupySubAccount(data) => execute::account::occupy_sub_account(deps, env, data),
        ExecuteMsg::FreeSubAccount(data) => execute::account::free_sub_account(deps, env, data),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    match msg {
        QueryMsg::QueryConfig(_) => to_binary(&query::account::query_config(config)?),
        QueryMsg::QueryOccupiedSubAccounts(data) => to_binary(
            &query::account::query_occupied_sub_accounts(deps, data, config)?,
        ),
        QueryMsg::QueryFreeSubAccounts(data) => to_binary(
            &query::account::query_free_sub_accounts(deps, data, config)?,
        ),
        QueryMsg::QueryFirstFreeSubAccount(_) => {
            to_binary(&query::account::query_first_free_sub_account(deps, config)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
}

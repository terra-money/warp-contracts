use crate::state::{CONFIG, FREE_SUB_ACCOUNTS, IN_USE_SUB_ACCOUNTS};
use crate::ContractError;
use account::{Config, ExecuteMsg, InstantiateMsg, QueryMsg, SubAccount, WithdrawAssetsMsg};
use controller::account::{AssetInfo, Cw721ExecuteMsg};
use cosmwasm_std::{
    entry_point, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Order, Response, StdResult, Uint128, Uint64, WasmMsg,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg};
use cw721::{Cw721QueryMsg, OwnerOfResponse};
use cw_storage_plus::Bound;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&msg.owner)?,
            warp_addr: info.sender,
        },
    )?;
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_addr", env.contract.address)
        .add_attribute("owner", msg.owner)
        .add_attribute("funds", serde_json_wasm::to_string(&info.funds)?)
        .add_attribute("cw_funds", serde_json_wasm::to_string(&msg.funds)?)
        .add_attribute("job_id", msg.job_id.unwrap_or(Uint64::zero()))
        .add_attribute("msgs", msg.msgs.unwrap_or("".to_string())))
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
            .add_attribute("job_id", data.job_id.unwrap_or(Uint64::zero()))
            .add_attribute("action", "generic")),
        ExecuteMsg::WithdrawAssets(data) => withdraw_assets(deps, env, info, data),
        ExecuteMsg::UpdateSubAccountFromFreeToInUse(data) => {
            // We do not add default account to in use sub accounts
            if data.sub_account == env.contract.address {
                return Ok(Response::new());
            }
            FREE_SUB_ACCOUNTS.remove(deps.storage, data.sub_account.clone());
            IN_USE_SUB_ACCOUNTS.update(deps.storage, data.sub_account.clone(), |s| match s {
                None => Ok(data.job_id.u64()),
                Some(_) => Err(ContractError::SubAccountAlreadyInUseError {}),
            })?;
            Ok(Response::new()
                .add_attribute("action", "add_in_use_sub_account")
                .add_attribute("sub_account", data.sub_account)
                .add_attribute("job_id", data.job_id))
        }
        ExecuteMsg::UpdateSubAccountFromInUseToFree(data) => {
            // We do not add default account to free sub accounts
            if data.sub_account == env.contract.address {
                return Ok(Response::new());
            }
            IN_USE_SUB_ACCOUNTS.remove(deps.storage, data.sub_account.clone());
            FREE_SUB_ACCOUNTS.update(deps.storage, data.sub_account.clone(), |s| match s {
                // value is a dummy data because there is no built in support for set in cosmwasm
                None => Ok(0),
                Some(_) => Err(ContractError::SubAccountAlreadyFreeError {}),
            })?;
            Ok(Response::new()
                .add_attribute("action", "free_in_use_sub_account")
                .add_attribute("sub_account", data.sub_account))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config => {
            let config = CONFIG.load(deps.storage)?;
            return to_binary(&config);
        }
        QueryMsg::QueryInUseSubAccounts(data) => {
            let sub_accounts = IN_USE_SUB_ACCOUNTS
                .range(
                    deps.storage,
                    data.start_after.map(Bound::exclusive),
                    None,
                    Order::Descending,
                )
                .take(data.limit.unwrap_or(0) as usize)
                .map(|item| {
                    item.map(|(k, v)| SubAccount {
                        addr: k,
                        job_id: Some(Uint64::from(v)),
                    })
                })
                .collect::<StdResult<Vec<SubAccount>>>()?;
            return to_binary(&sub_accounts);
        }
        QueryMsg::QueryFreeSubAccounts(data) => {
            let sub_accounts = FREE_SUB_ACCOUNTS
                .range(
                    deps.storage,
                    data.start_after.map(Bound::exclusive),
                    None,
                    Order::Descending,
                )
                .take(data.limit.unwrap_or(0) as usize)
                .map(|item| {
                    item.map(|(k, _)| SubAccount {
                        addr: k,
                        job_id: Option::None,
                    })
                })
                .collect::<StdResult<Vec<SubAccount>>>()?;
            return to_binary(&sub_accounts);
        }
        QueryMsg::QueryFirstFreeSubAccount {} => {
            let sub_account = FREE_SUB_ACCOUNTS
                .range(deps.storage, None, None, Order::Ascending)
                .next()
                .map(|item| {
                    item.map(|(k, _)| SubAccount {
                        addr: k,
                        job_id: Option::None,
                    })
                })
                .unwrap()?;
            return to_binary(&sub_account);
        }
    }
}

pub fn migrate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn withdraw_assets(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: WithdrawAssetsMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner && info.sender != config.warp_addr {
        return Err(ContractError::Unauthorized {});
    }

    let mut withdraw_msgs: Vec<CosmosMsg> = vec![];

    for asset_info in &data.asset_infos {
        match asset_info {
            AssetInfo::Native(denom) => {
                let withdraw_native_msg =
                    withdraw_asset_native(deps.as_ref(), env.clone(), &config.owner, denom)?;

                match withdraw_native_msg {
                    None => {}
                    Some(msg) => withdraw_msgs.push(msg),
                }
            }
            AssetInfo::Cw20(addr) => {
                let withdraw_cw20_msg =
                    withdraw_asset_cw20(deps.as_ref(), env.clone(), &config.owner, addr)?;

                match withdraw_cw20_msg {
                    None => {}
                    Some(msg) => withdraw_msgs.push(msg),
                }
            }
            AssetInfo::Cw721(addr, token_id) => {
                let withdraw_cw721_msg =
                    withdraw_asset_cw721(deps.as_ref(), &config.owner, addr, token_id)?;
                match withdraw_cw721_msg {
                    None => {}
                    Some(msg) => withdraw_msgs.push(msg),
                }
            }
        }
    }

    Ok(Response::new()
        .add_messages(withdraw_msgs)
        .add_attribute("action", "withdraw_assets")
        .add_attribute("assets", serde_json_wasm::to_string(&data.asset_infos)?))
}

fn withdraw_asset_native(
    deps: Deps,
    env: Env,
    owner: &Addr,
    denom: &String,
) -> StdResult<Option<CosmosMsg>> {
    let amount = deps.querier.query_balance(env.contract.address, denom)?;

    let res = if amount.amount > Uint128::zero() {
        Some(CosmosMsg::Bank(BankMsg::Send {
            to_address: owner.to_string(),
            amount: vec![amount],
        }))
    } else {
        None
    };

    Ok(res)
}

fn withdraw_asset_cw20(
    deps: Deps,
    env: Env,
    owner: &Addr,
    token: &Addr,
) -> StdResult<Option<CosmosMsg>> {
    let amount: BalanceResponse = deps.querier.query_wasm_smart(
        token.to_string(),
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;

    let res = if amount.balance > Uint128::zero() {
        Some(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: owner.to_string(),
                amount: amount.balance,
            })?,
            funds: vec![],
        }))
    } else {
        None
    };

    Ok(res)
}

fn withdraw_asset_cw721(
    deps: Deps,
    owner: &Addr,
    token: &Addr,
    token_id: &String,
) -> StdResult<Option<CosmosMsg>> {
    let owner_query: OwnerOfResponse = deps.querier.query_wasm_smart(
        token.to_string(),
        &Cw721QueryMsg::OwnerOf {
            token_id: token_id.to_string(),
            include_expired: None,
        },
    )?;

    let res = if owner_query.owner == *owner {
        Some(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token.to_string(),
            msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: owner.to_string(),
                token_id: token_id.to_string(),
            })?,
            funds: vec![],
        }))
    } else {
        None
    };

    Ok(res)
}

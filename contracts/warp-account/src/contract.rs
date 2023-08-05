use crate::state::CONFIG;
use crate::ContractError;
use account::{Config, ExecuteMsg, InstantiateMsg, QueryMsg, WithdrawAssetsMsg};
use controller::account::{AssetInfo, Cw721ExecuteMsg};
use controller::account::{Fund, FundTransferMsgs, TransferFromMsg, TransferNftMsg};
use cosmwasm_std::{
    entry_point, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128, WasmMsg,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg};
use cw721::{Cw721QueryMsg, OwnerOfResponse};

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

    let cw_funds_vec = match msg.funds.clone() {
        None => {
            vec![]
        }
        Some(funds) => funds,
    };

    let mut fund_msgs_vec: Vec<CosmosMsg> = vec![];

    if !info.funds.is_empty() {
        fund_msgs_vec.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: env.contract.address.to_string(),
            amount: info.funds.clone(),
        }))
    }

    for cw_fund in &cw_funds_vec {
        fund_msgs_vec.push(CosmosMsg::Wasm(match cw_fund {
            Fund::Cw20(cw20_fund) => WasmMsg::Execute {
                contract_addr: deps
                    .api
                    .addr_validate(&cw20_fund.contract_addr)?
                    .to_string(),
                msg: to_binary(&FundTransferMsgs::TransferFrom(TransferFromMsg {
                    owner: msg.owner.clone().to_string(),
                    recipient: env.contract.address.clone().to_string(),
                    amount: cw20_fund.amount,
                }))?,
                funds: vec![],
            },
            Fund::Cw721(cw721_fund) => WasmMsg::Execute {
                contract_addr: deps
                    .api
                    .addr_validate(&cw721_fund.contract_addr)?
                    .to_string(),
                msg: to_binary(&FundTransferMsgs::TransferNft(TransferNftMsg {
                    recipient: env.contract.address.clone().to_string(),
                    token_id: cw721_fund.token_id.clone(),
                }))?,
                funds: vec![],
            },
        }));
    }

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_addr", env.contract.address)
        .add_attribute("owner", msg.owner)
        .add_attribute("funds", serde_json_wasm::to_string(&info.funds)?)
        .add_attribute("cw_funds", serde_json_wasm::to_string(&msg.funds)?)
        .add_messages(fund_msgs_vec)
        .add_messages(msg.msgs.unwrap_or(vec![])))
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
        ExecuteMsg::WithdrawAssets(data) => withdraw_assets(deps, env, info, data),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    to_binary("")
}

pub fn migrate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.warp_addr {
        return Err(ContractError::Unauthorized {});
    }
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

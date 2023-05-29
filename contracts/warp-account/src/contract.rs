use crate::state::CONFIG;
use crate::ContractError;
use account::{AssetInfo, Config, ExecuteMsg, InstantiateMsg, QueryMsg, WithdrawAssetsMsg};
use controller::account::{Cw721ExecuteMsg};
use cosmwasm_std::{
    entry_point, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, WasmMsg,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg};

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
        .add_attribute("cw_funds", serde_json_wasm::to_string(&msg.funds)?))
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
        ExecuteMsg::Generic(data) => {
            Ok(Response::new()
                .add_messages(data.msgs)
                .add_attribute("action", "generic")
            )
        },
        ExecuteMsg::WithdrawAssets(data) => withdraw_assets(deps, env, info, data)
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
    _info: MessageInfo,
    data: WithdrawAssetsMsg,
) -> Result<Response, ContractError> {
    let owner = CONFIG.load(deps.storage)?.owner;
    let mut withdraw_msgs = vec![];

    for asset_info in &data.asset_infos {
        match asset_info {
            AssetInfo::Native(denom) => withdraw_msgs.push(withdraw_asset_native(
                deps.as_ref(),
                env.clone(),
                &owner,
                &denom,
            )?),
            AssetInfo::Cw20(addr) => withdraw_msgs.push(withdraw_asset_cw20(
                deps.as_ref(),
                env.clone(),
                &owner,
                &addr,
            )?),
            AssetInfo::Cw721(addr, token_id) => withdraw_msgs.push(withdraw_asset_cw721(
                deps.as_ref(),
                &addr,
                &owner,
                &token_id,
            )?),
        }
    }

    Ok(Response::new()
        .add_messages(withdraw_msgs)
        .add_attribute("action", "withdraw_assets")
        .add_attribute("assets", serde_json_wasm::to_string(&data.asset_infos)?))
}

fn withdraw_asset_native(deps: Deps, env: Env, owner: &Addr, denom: &str) -> StdResult<CosmosMsg> {
    let amount = deps.querier.query_balance(env.contract.address, denom)?;

    Ok(CosmosMsg::Bank(BankMsg::Send {
        to_address: owner.to_string(),
        amount: vec![amount],
    }))
}

fn withdraw_asset_cw20(deps: Deps, env: Env, owner: &Addr, token: &Addr) -> StdResult<CosmosMsg> {
    let amount = deps
        .querier
        .query_wasm_smart::<BalanceResponse>(
            token.to_string(),
            &cw20::Cw20QueryMsg::Balance {
                address: env.contract.address.to_string(),
            },
        )?
        .balance;

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: owner.to_string(),
            amount,
        })?,
        funds: vec![],
    }))
}

fn withdraw_asset_cw721(
    _deps: Deps,
    owner: &Addr,
    token: &Addr,
    token_id: &str,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token.to_string(),
        msg: to_binary(&Cw721ExecuteMsg::TransferNft {
            recipient: owner.to_string(),
            token_id: token_id.to_string(),
        })?,
        funds: vec![],
    }))
}

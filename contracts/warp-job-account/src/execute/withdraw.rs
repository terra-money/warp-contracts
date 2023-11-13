use cosmwasm_std::{
    to_binary, Addr, BankMsg, CosmosMsg, Deps, Env, Response, StdResult, Uint128, WasmMsg,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg};
use cw721::{Cw721QueryMsg, OwnerOfResponse};

use crate::ContractError;
use controller::account::{AssetInfo, Cw721ExecuteMsg, WithdrawAssetsMsg};
use job_account::Config;

pub fn withdraw_assets(
    deps: Deps,
    env: Env,
    data: WithdrawAssetsMsg,
    config: Config,
) -> Result<Response, ContractError> {
    let mut withdraw_msgs: Vec<CosmosMsg> = vec![];

    for asset_info in &data.asset_infos {
        match asset_info {
            AssetInfo::Native(denom) => {
                let withdraw_native_msg =
                    withdraw_asset_native(deps, env.clone(), &config.owner, denom)?;

                match withdraw_native_msg {
                    None => {}
                    Some(msg) => withdraw_msgs.push(msg),
                }
            }
            AssetInfo::Cw20(addr) => {
                let withdraw_cw20_msg =
                    withdraw_asset_cw20(deps, env.clone(), &config.owner, addr)?;

                match withdraw_cw20_msg {
                    None => {}
                    Some(msg) => withdraw_msgs.push(msg),
                }
            }
            AssetInfo::Cw721(addr, token_id) => {
                let withdraw_cw721_msg = withdraw_asset_cw721(deps, &config.owner, addr, token_id)?;
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

use crate::state::{ACCOUNTS, CONFIG};
use crate::ContractError;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, ReplyOn, Response,
    StdResult, SubMsg, Uint128, WasmMsg,
};
use cw20::BalanceResponse;
use cw20::Cw20ExecuteMsg;
use cw721::Cw721ExecuteMsg;
use warp_protocol::controller::account::{Account, AssetInfo, WithdrawAssetMsg};
use warp_protocol::controller::account::CreateAccountMsg;

pub fn create_account(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data: CreateAccountMsg
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let item = ACCOUNTS()
        .idx
        .account
        .item(deps.storage, info.sender.clone());

    if item?.is_some() {
        return Err(ContractError::AccountCannotCreateAccount {});
    }

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
                funds: data.funds,
            })?,
            funds: info.funds,
            label: info.sender.to_string(),
        }),
        gas_limit: None,
        reply_on: ReplyOn::Always,
    };

    Ok(Response::new()
        .add_attribute("action", "create_account")
        .add_submessage(submsg))
}

pub fn withdraw_asset(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data: WithdrawAssetMsg,
) -> Result<Response, ContractError> {
    let q = ACCOUNTS()
        .idx
        .account
        .item(deps.storage, info.sender.clone())?;

    let account = match q {
        None => ACCOUNTS()
            .load(deps.storage, info.sender)
            .map_err(|_e| ContractError::AccountDoesNotExist {})?,
        Some(q) => q.1,
    };

    match data.asset_info {
        AssetInfo::Native(denom) => withdraw_asset_native(deps, &account, &denom),
        AssetInfo::Cw20(token) => withdraw_asset_cw20(deps, &account, &token),
        AssetInfo::Cw721(token, token_id) => {
            withdraw_asset_cw721(deps, &account, &token, &token_id)
        }
    }
}

fn withdraw_asset_native(
    deps: DepsMut,
    account: &Account,
    denom: &str,
) -> Result<Response, ContractError> {
    let amount = query_native_token_balance(deps, &account.account, denom).unwrap();

    let msgs = vec![WasmMsg::Execute {
        contract_addr: account.account.to_string(),
        msg: to_binary(&warp_protocol::account::ExecuteMsg {
            msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                to_address: account.owner.to_string(),
                amount: vec![Coin::new(amount.u128(), denom)],
            })],
        })?,
        funds: vec![],
    }];

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "withdraw_asset")
        .add_attribute("amount", amount)
        .add_attribute("asset", denom))
}

fn withdraw_asset_cw20(
    deps: DepsMut,
    account: &Account,
    token: &Addr,
) -> Result<Response, ContractError> {
    let amount = query_cw20_balance(deps, &account.account, token).unwrap();

    let msgs = vec![WasmMsg::Execute {
        contract_addr: account.account.to_string(),
        msg: to_binary(&warp_protocol::account::ExecuteMsg {
            msgs: vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: token.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: account.owner.to_string(),
                    amount: amount,
                })?,
                funds: vec![],
            })],
        })?,
        funds: vec![],
    }];

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "withdraw_asset")
        .add_attribute("amount", amount)
        .add_attribute("asset", token))
}

fn withdraw_asset_cw721(
    _deps: DepsMut,
    account: &Account,
    token: &Addr,
    token_id: &str,
) -> Result<Response, ContractError> {
    let msgs = vec![WasmMsg::Execute {
        contract_addr: account.account.to_string(),
        msg: to_binary(&warp_protocol::account::ExecuteMsg {
            msgs: vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: token.to_string(),
                msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                    recipient: account.owner.to_string(),
                    token_id: token_id.to_string(),
                })?,
                funds: vec![],
            })],
        })?,
        funds: vec![],
    }];

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "withdraw_asset")
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("asset", token))
}

fn query_native_token_balance(
    deps: DepsMut,
    wallet_address: &Addr,
    denom: &str,
) -> StdResult<Uint128> {
    let all_balances = deps.querier.query_all_balances(wallet_address)?;

    let balance = all_balances
        .into_iter()
        .find(|coin| coin.denom == denom)
        .unwrap()
        .amount;

    Ok(balance)
}

fn query_cw20_balance(
    deps: DepsMut,
    wallet_address: &Addr,
    cw20_token: &Addr,
) -> StdResult<Uint128> {
    let response: BalanceResponse = deps.querier.query_wasm_smart(
        cw20_token.to_string(),
        &cw20::Cw20QueryMsg::Balance {
            address: wallet_address.to_string(),
        },
    )?;

    Ok(response.balance)
}

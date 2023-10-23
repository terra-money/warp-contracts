use cosmwasm_std::{to_binary, BankMsg, Coin, CosmosMsg, Uint128, Uint64, WasmMsg};

use account::{FreeSubAccountMsg, GenericMsg, OccupySubAccountMsg, WithdrawAssetsMsg};
use controller::account::{AssetInfo, CwFund, FundTransferMsgs, TransferFromMsg, TransferNftMsg};

pub fn build_instantiate_warp_main_account_msg(
    job_id: Uint64,
    admin_addr: String,
    code_id: u64,
    account_owner: String,
    native_funds: Vec<Coin>,
    cw_funds: Option<Vec<CwFund>>,
    msgs: Option<Vec<CosmosMsg>>,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(admin_addr),
        code_id,
        msg: to_binary(&account::InstantiateMsg {
            owner: account_owner.clone(),
            job_id,
            is_sub_account: false,
            main_account_addr: None,
            native_funds: native_funds.clone(),
            cw_funds: cw_funds.unwrap_or(vec![]),
            msgs: msgs.unwrap_or(vec![]),
        })
        .unwrap(),
        // Only send native funds to sub account
        funds: vec![],
        label: format!("warp main account, owner: {}", account_owner),
    })
}

#[allow(clippy::too_many_arguments)]
pub fn build_instantiate_warp_sub_account_msg(
    job_id: Uint64,
    admin_addr: String,
    code_id: u64,
    account_owner: String,
    main_account_addr: String,
    native_funds: Vec<Coin>,
    cw_funds: Option<Vec<CwFund>>,
    msgs: Option<Vec<CosmosMsg>>,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(admin_addr),
        code_id,
        msg: to_binary(&account::InstantiateMsg {
            owner: account_owner.clone(),
            job_id,
            is_sub_account: true,
            main_account_addr: Some(main_account_addr.clone()),
            native_funds: native_funds.clone(),
            cw_funds: cw_funds.unwrap_or(vec![]),
            msgs: msgs.unwrap_or(vec![]),
        })
        .unwrap(),
        funds: native_funds,
        label: format!(
            "warp sub account, main account: {}, owner: {}",
            main_account_addr, account_owner,
        ),
    })
}

pub fn build_free_sub_account_msg(
    main_account_addr: String,
    sub_account_addr: String,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: main_account_addr,
        msg: to_binary(&account::ExecuteMsg::FreeSubAccount(FreeSubAccountMsg {
            sub_account_addr,
        }))
        .unwrap(),
        funds: vec![],
    })
}

pub fn build_occupy_sub_account_msg(
    main_account_addr: String,
    sub_account_addr: String,
    job_id: Uint64,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: main_account_addr,
        msg: to_binary(&account::ExecuteMsg::OccupySubAccount(
            OccupySubAccountMsg {
                sub_account_addr,
                job_id,
            },
        ))
        .unwrap(),
        funds: vec![],
    })
}

pub fn build_transfer_cw20_msg(
    cw20_token_contract_addr: String,
    owner_addr: String,
    recipient_addr: String,
    amount: Uint128,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: cw20_token_contract_addr,
        msg: to_binary(&FundTransferMsgs::TransferFrom(TransferFromMsg {
            owner: owner_addr,
            recipient: recipient_addr,
            amount,
        }))
        .unwrap(),
        funds: vec![],
    })
}

pub fn build_transfer_cw721_msg(
    cw721_token_contract_addr: String,
    recipient_addr: String,
    token_id: String,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: cw721_token_contract_addr,
        msg: to_binary(&FundTransferMsgs::TransferNft(TransferNftMsg {
            recipient: recipient_addr,
            token_id,
        }))
        .unwrap(),
        funds: vec![],
    })
}

pub fn build_transfer_native_funds_msg(
    recipient_addr: String,
    native_funds: Vec<Coin>,
) -> CosmosMsg {
    CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient_addr,
        amount: native_funds,
    })
}

pub fn build_account_execute_generic_msgs(
    account_addr: String,
    cosmos_msgs_for_account_to_execute: Vec<CosmosMsg>,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: account_addr,
        msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {
            msgs: cosmos_msgs_for_account_to_execute,
        }))
        .unwrap(),
        funds: vec![],
    })
}

pub fn build_account_withdraw_assets_msg(
    account_addr: String,
    assets_to_withdraw: Vec<AssetInfo>,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: account_addr,
        msg: to_binary(&account::ExecuteMsg::WithdrawAssets(WithdrawAssetsMsg {
            asset_infos: assets_to_withdraw,
        }))
        .unwrap(),
        funds: vec![],
    })
}

use cosmwasm_std::{to_binary, BankMsg, Coin, CosmosMsg, Uint128, Uint64, WasmMsg};

use controller::account::{AssetInfo, CwFund, FundTransferMsgs, TransferFromMsg, TransferNftMsg};
use job_account::{GenericMsg, WithdrawAssetsMsg};
use job_account_tracker::{FreeAccountMsg, TakeAccountMsg};

pub fn build_instantiate_warp_job_account_tracker_msg(
    admin_addr: String,
    code_id: u64,
    account_owner: String,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(admin_addr),
        code_id,
        msg: to_binary(&job_account_tracker::InstantiateMsg {
            owner: account_owner.clone(),
        })
        .unwrap(),
        funds: vec![],
        label: format!("warp account tracker, owner: {}", account_owner),
    })
}

#[allow(clippy::too_many_arguments)]
pub fn build_instantiate_warp_account_msg(
    job_id: Uint64,
    admin_addr: String,
    code_id: u64,
    account_owner: String,
    job_account_tracker_addr: String,
    native_funds: Vec<Coin>,
    cw_funds: Option<Vec<CwFund>>,
    msgs: Option<Vec<CosmosMsg>>,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(admin_addr),
        code_id,
        msg: to_binary(&job_account::InstantiateMsg {
            owner: account_owner.clone(),
            job_id,
            job_account_tracker_addr,
            native_funds: native_funds.clone(),
            cw_funds: cw_funds.unwrap_or(vec![]),
            msgs: msgs.unwrap_or(vec![]),
        })
        .unwrap(),
        funds: native_funds,
        label: format!("warp account, owner: {}", account_owner,),
    })
}

pub fn build_free_account_msg(job_account_tracker_addr: String, account_addr: String) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: job_account_tracker_addr,
        msg: to_binary(&job_account_tracker::ExecuteMsg::FreeAccount(
            FreeAccountMsg { account_addr },
        ))
        .unwrap(),
        funds: vec![],
    })
}

pub fn build_taken_account_msg(
    job_account_tracker_addr: String,
    account_addr: String,
    job_id: Uint64,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: job_account_tracker_addr,
        msg: to_binary(&job_account_tracker::ExecuteMsg::TakeAccount(
            TakeAccountMsg {
                account_addr,
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
        msg: to_binary(&job_account::ExecuteMsg::Generic(GenericMsg {
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
        msg: to_binary(&job_account::ExecuteMsg::WithdrawAssets(
            WithdrawAssetsMsg {
                asset_infos: assets_to_withdraw,
            },
        ))
        .unwrap(),
        funds: vec![],
    })
}

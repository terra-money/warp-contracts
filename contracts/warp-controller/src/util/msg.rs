use cosmwasm_std::{to_binary, BankMsg, Coin, CosmosMsg, Uint128, Uint64, WasmMsg};

use account_tracker::{
    AddFundingAccountMsg, FreeAccountMsg, FreeFundingAccountMsg, TakeAccountMsg,
    TakeFundingAccountMsg,
};
use controller::account::{
    AssetInfo, CwFund, FundTransferMsgs, TransferFromMsg, TransferNftMsg, WarpMsg, WarpMsgs,
    WithdrawAssetsMsg,
};

#[allow(clippy::too_many_arguments)]
pub fn build_instantiate_account_tracker_msg(
    admin_addr: String,
    controller_addr: String,
    code_id: u64,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(admin_addr.clone()),
        code_id,
        msg: to_binary(&account_tracker::InstantiateMsg {
            admin: admin_addr,
            warp_addr: controller_addr,
        })
        .unwrap(),
        funds: vec![],
        label: "warp job account tracker".to_string(),
    })
}

#[allow(clippy::too_many_arguments)]
pub fn build_instantiate_warp_account_msg(
    job_id: Uint64,
    admin_addr: String,
    code_id: u64,
    account_owner: String,
    native_funds: Vec<Coin>,
    cw_funds: Option<Vec<CwFund>>,
    msgs: Option<Vec<WarpMsg>>,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(admin_addr),
        code_id,
        msg: to_binary(&account::InstantiateMsg {
            owner: account_owner.clone(),
            job_id,
            native_funds: native_funds.clone(),
            cw_funds: cw_funds.unwrap_or(vec![]),
            msgs: msgs.unwrap_or(vec![]),
        })
        .unwrap(),
        funds: native_funds,
        label: format!("warp account, owner: {}", account_owner,),
    })
}

pub fn build_free_account_msg(
    account_tracker_addr: String,
    account_owner_addr: String,
    account_addr: String,
    last_job_id: Uint64,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: account_tracker_addr,
        msg: to_binary(&account_tracker::ExecuteMsg::FreeAccount(FreeAccountMsg {
            account_owner_addr,
            account_addr,
            last_job_id,
        }))
        .unwrap(),
        funds: vec![],
    })
}

pub fn build_taken_account_msg(
    account_tracker_addr: String,
    account_owner_addr: String,
    account_addr: String,
    job_id: Uint64,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: account_tracker_addr,
        msg: to_binary(&account_tracker::ExecuteMsg::TakeAccount(TakeAccountMsg {
            account_owner_addr,
            account_addr,
            job_id,
        }))
        .unwrap(),
        funds: vec![],
    })
}

pub fn build_free_funding_account_msg(
    account_tracker_addr: String,
    account_owner_addr: String,
    account_addr: String,
    job_id: Uint64,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: account_tracker_addr,
        msg: to_binary(&account_tracker::ExecuteMsg::FreeFundingAccount(
            FreeFundingAccountMsg {
                account_owner_addr,
                account_addr,
                job_id,
            },
        ))
        .unwrap(),
        funds: vec![],
    })
}

pub fn build_take_funding_account_msg(
    account_tracker_addr: String,
    account_owner_addr: String,
    account_addr: String,
    job_id: Uint64,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: account_tracker_addr,
        msg: to_binary(&account_tracker::ExecuteMsg::TakeFundingAccount(
            TakeFundingAccountMsg {
                account_owner_addr,
                account_addr,
                job_id,
            },
        ))
        .unwrap(),
        funds: vec![],
    })
}

pub fn build_add_funding_account_msg(
    account_tracker_addr: String,
    account_owner_addr: String,
    account_addr: String,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: account_tracker_addr,
        msg: to_binary(&account_tracker::ExecuteMsg::AddFundingAccount(
            AddFundingAccountMsg {
                account_owner_addr,
                account_addr,
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
    build_account_execute_warp_msgs(
        account_addr,
        cosmos_msgs_for_account_to_execute
            .into_iter()
            .map(WarpMsg::Generic)
            .collect(),
    )
}

pub fn build_account_execute_warp_msgs(
    account_addr: String,
    warp_msgs_for_account_to_execute: Vec<WarpMsg>,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: account_addr,
        msg: to_binary(&account::ExecuteMsg::WarpMsgs(WarpMsgs {
            msgs: warp_msgs_for_account_to_execute,
            job_id: None,
        }))
        .unwrap(),
        funds: vec![],
    })
}

pub fn build_account_withdraw_assets_msg(
    account_addr: String,
    assets_to_withdraw: Vec<AssetInfo>,
) -> CosmosMsg {
    build_account_execute_warp_msgs(
        account_addr,
        vec![WarpMsg::WithdrawAssets(WithdrawAssetsMsg {
            asset_infos: assets_to_withdraw,
        })],
    )
}

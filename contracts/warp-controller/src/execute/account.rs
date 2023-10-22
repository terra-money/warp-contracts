use crate::state::{ACCOUNTS, CONFIG};
use crate::ContractError;
use controller::account::{Fund, FundTransferMsgs, TransferFromMsg, TransferNftMsg};

use cosmwasm_std::{
    to_binary, BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, ReplyOn, Response, SubMsg, WasmMsg,
};

// pub fn create_account(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     data: CreateAccountMsg,
// ) -> Result<Response, ContractError> {
//     let config = CONFIG.load(deps.storage)?;

//     let item = ACCOUNTS()
//         .idx
//         .account
//         .item(deps.storage, info.sender.clone());

//     if item?.is_some() {
//         return Err(ContractError::AccountCannotCreateAccount {});
//     }

//     if ACCOUNTS().has(deps.storage, info.sender.clone()) {
//         let account = ACCOUNTS().load(deps.storage, info.sender.clone())?;

//         let cw_funds_vec = match data.funds {
//             None => {
//                 vec![]
//             }
//             Some(funds) => funds,
//         };

//         let mut msgs_vec: Vec<CosmosMsg> = vec![];

//         if !info.funds.is_empty() {
//             msgs_vec.push(CosmosMsg::Bank(BankMsg::Send {
//                 to_address: account.account.to_string(),
//                 amount: info.funds.clone(),
//             }))
//         }

//         for cw_fund in &cw_funds_vec {
//             msgs_vec.push(CosmosMsg::Wasm(match cw_fund {
//                 Fund::Cw20(cw20_fund) => WasmMsg::Execute {
//                     contract_addr: deps
//                         .api
//                         .addr_validate(&cw20_fund.contract_addr)?
//                         .to_string(),
//                     msg: to_binary(&FundTransferMsgs::TransferFrom(TransferFromMsg {
//                         owner: info.sender.clone().to_string(),
//                         recipient: account.account.clone().to_string(),
//                         amount: cw20_fund.amount,
//                     }))?,
//                     funds: vec![],
//                 },
//                 Fund::Cw721(cw721_fund) => WasmMsg::Execute {
//                     contract_addr: deps
//                         .api
//                         .addr_validate(&cw721_fund.contract_addr)?
//                         .to_string(),
//                     msg: to_binary(&FundTransferMsgs::TransferNft(TransferNftMsg {
//                         recipient: account.account.clone().to_string(),
//                         token_id: cw721_fund.token_id.clone(),
//                     }))?,
//                     funds: vec![],
//                 },
//             }))
//         }

//         return Ok(Response::new()
//             .add_attribute("action", "create_account")
//             .add_attribute("owner", account.owner)
//             .add_attribute("account_address", account.account)
//             .add_messages(msgs_vec));
//     }

//     let submsg = SubMsg {
//         id: 0,
//         msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
//             admin: Some(env.contract.address.to_string()),
//             code_id: config.warp_account_code_id.u64(),
//             msg: to_binary(&account::InstantiateMsg {
//                 owner: info.sender.to_string(),
//                 funds: data.funds,
//                 msgs: data.msgs,
//                 is_sub_account: Some(false),
//                 main_account_addr: None,
//             })?,
//             funds: info.funds,
//             label: info.sender.to_string(),
//         }),
//         gas_limit: None,
//         reply_on: ReplyOn::Always,
//     };

//     Ok(Response::new()
//         .add_attribute("action", "create_account")
//         .add_submessage(submsg))
// }

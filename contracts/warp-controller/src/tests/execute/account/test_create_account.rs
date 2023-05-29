use crate::execute::account::create_account;
use crate::state::ACCOUNTS;
use crate::ContractError;
use controller::account::{Account, CreateAccountMsg, Cw20Fund, Fund};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, ReplyOn, Response, SubMsg, Uint128, WasmMsg,
};

#[test]
fn test_create_account_success() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("user1", &[]);

    let response = create_account(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        CreateAccountMsg {
            funds: Some(vec![Fund::Cw20(Cw20Fund {
                contract_addr: "uatom".to_string(),
                amount: Uint128::new(1000),
            })]),
        },
    )
    .unwrap();

    // Verify that the response is as expected
    assert_eq!(
        response,
        Response::new()
            .add_attribute("action", "create_account")
            .add_submessage(SubMsg {
                id: 0,
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    admin: None,
                    code_id: 123, // Replace with the actual code ID
                    msg: to_binary(&account::InstantiateMsg {
                        owner: "user1".to_string(),
                        funds: Some(vec![Fund::Cw20(Cw20Fund {
                            contract_addr: "uatom".to_string(),
                            amount: Uint128::new(1000)
                        })]),
                    })
                    .unwrap(),
                    funds: info.funds.clone(),
                    label: "user1".to_string(),
                }),
                gas_limit: None,
                reply_on: ReplyOn::Always,
            })
    );
}

#[test]
fn test_create_account_existing_account() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("user1", &[]);

    // Create an existing account for the sender
    ACCOUNTS()
        .save(
            deps.as_mut().storage,
            Addr::unchecked("user1".to_string()),
            &Account {
                owner: Addr::unchecked("user1"),
                account: Addr::unchecked("account1".to_string()),
            },
        )
        .unwrap();

    let error = create_account(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        CreateAccountMsg {
            funds: Some(vec![Fund::Cw20(Cw20Fund {
                contract_addr: "uatom".to_string(),
                amount: Uint128::new(1000),
            })]),
        },
    )
    .unwrap_err();

    // Verify that the error is as expected
    assert_eq!(error, ContractError::AccountCannotCreateAccount {});
}

#[test]
fn test_create_account_already_created() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("user1", &[]);

    // Mark the account as already created
    ACCOUNTS()
        .save(
            deps.as_mut().storage,
            Addr::unchecked("user1".to_string()),
            &Account {
                owner: Addr::unchecked("user1".to_string()),
                account: Addr::unchecked("account1".to_string()),
            },
        )
        .unwrap();

    let response = create_account(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        CreateAccountMsg {
            funds: Some(vec![Fund::Cw20(Cw20Fund {
                contract_addr: "uatom".to_string(),
                amount: Uint128::new(1000),
            })]),
        },
    )
    .unwrap();

    // Verify that the response is as expected
    assert_eq!(
        response,
        Response::new()
            .add_attribute("action", "create_account")
            .add_attribute("owner", "user1")
            .add_attribute("account_address", "account1")
    );
}

// use crate::tests::helpers::{create_warp_account, instantiate_warp};
// use crate::ContractError;
// use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
// use cosmwasm_std::{
//     coin, to_binary, CosmosMsg, ReplyOn, Response, StdError, SubMsg, Uint128, Uint64, WasmMsg,
// };
//
// #[test]
// fn test_create_account_success() {
//     let mut deps = mock_dependencies();
//     let env = mock_env();
//     let info = mock_info("vlad", &vec![]);
//
//     let _instantiate_res = instantiate_warp(
//         deps.as_mut(),
//         env.clone(),
//         info.clone(),
//         Some(info.sender.to_string()),
//         Some(info.sender.to_string()),
//         Uint64::new(0),
//         Uint128::new(0),
//         Uint64::new(0),
//         Uint64::new(0),
//         Default::default(),
//         Default::default(),
//         Default::default(),
//         Default::default(),
//         Default::default(),
//         Default::default(),
//     )
//     .unwrap();
//
//     let (create_account_res, reply_res) =
//         create_warp_account(&mut deps, env.clone(), info.clone(), Uint64::new(0));
//
//     assert_eq!(
//         create_account_res.unwrap(),
//         Response::new()
//             .add_attribute("action", "create_account")
//             .add_submessage(SubMsg {
//                 id: 0,
//                 msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
//                     admin: None,
//                     code_id: 0,
//                     msg: to_binary(&account::InstantiateMsg {
//                         owner: info.sender.to_string(),
//                         funds: None,
//                     })
//                     .unwrap(),
//                     funds: vec![],
//                     label: info.sender.to_string(),
//                 }),
//                 gas_limit: None,
//                 reply_on: ReplyOn::Always,
//             })
//     );
//
//     assert_eq!(
//         reply_res.unwrap(),
//         Response::new()
//             .add_attribute("action", "save_account")
//             .add_attribute("owner", "terra1vladvladvladvladvladvladvladvladvl1000")
//             .add_attribute(
//                 "account_address",
//                 "terra1vladvladvladvladvladvladvladvladvl2000"
//             )
//             .add_attribute("funds", "[]")
//             .add_attribute("cw_funds", "[]")
//     )
// }
//
// #[test]
// fn test_create_account_exists() {
//     let mut deps = mock_dependencies();
//     let env = mock_env();
//     let info = mock_info("vlad", &vec![]);
//
//     let _instantiate_res = instantiate_warp(
//         deps.as_mut(),
//         env.clone(),
//         info.clone(),
//         Some(info.sender.to_string()),
//         Some(info.sender.to_string()),
//         Uint64::new(0),
//         Uint128::new(0),
//         Uint64::new(0),
//         Uint64::new(0),
//         Default::default(),
//         Default::default(),
//         Default::default(),
//         Default::default(),
//         Default::default(),
//         Default::default(),
//     )
//     .unwrap();
//
//     let (_create_account_res_first, _reply_res_first) =
//         create_warp_account(&mut deps, env.clone(), info.clone(), Uint64::new(0));
//     let (create_account_res, reply_res) =
//         create_warp_account(&mut deps, env.clone(), info.clone(), Uint64::new(0));
//
//     assert_eq!(
//         create_account_res.unwrap(),
//         Response::new()
//             .add_attribute("action", "create_account")
//             .add_submessage(SubMsg {
//                 id: 0,
//                 msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
//                     admin: None,
//                     code_id: 0,
//                     msg: to_binary(&account::InstantiateMsg {
//                         owner: info.sender.to_string(),
//                         funds: None,
//                     })
//                     .unwrap(),
//                     funds: vec![],
//                     label: info.sender.to_string(),
//                 }),
//                 gas_limit: None,
//                 reply_on: ReplyOn::Always,
//             })
//     );
//
//     assert_eq!(
//         reply_res.unwrap_err(),
//         ContractError::AccountAlreadyExists {}
//     )
// }
//
// #[test]
// fn test_create_account_by_account() {
//     let mut deps = mock_dependencies();
//     let env = mock_env();
//     let info = mock_info("vlad", &vec![coin(100, "uluna")]);
//
//     let _instantiate_res = instantiate_warp(
//         deps.as_mut(),
//         env.clone(),
//         info.clone(),
//         Some(info.sender.to_string()),
//         Some(info.sender.to_string()),
//         Uint64::new(0),
//         Uint128::new(0),
//         Uint64::new(0),
//         Uint64::new(0),
//         Default::default(),
//         Default::default(),
//         Default::default(),
//         Default::default(),
//         Default::default(),
//         Default::default(),
//     )
//     .unwrap();
//
//     let (_create_account_res_first, reply_res_first) =
//         create_warp_account(&mut deps, env.clone(), info.clone(), Uint64::new(0));
//
//     // Get address of warp account just created and assign it as the sender of next create_account call
//     let reply_res_first_clone = reply_res_first.unwrap().clone();
//     let attr_warp_account_address = reply_res_first_clone
//         .attributes
//         .iter()
//         .find(|attr| attr.key == "account_address")
//         .ok_or_else(|| StdError::generic_err("cannot find `account_address` attribute"))
//         .unwrap();
//     let info = mock_info(
//         attr_warp_account_address.value.as_str(),
//         &vec![coin(100, "uluna")],
//     );
//     let (create_account_res, _reply_res) =
//         create_warp_account(&mut deps, env.clone(), info.clone(), Uint64::new(0));
//
//     assert_eq!(
//         create_account_res.unwrap_err(),
//         ContractError::AccountCannotCreateAccount {}
//     );
// }

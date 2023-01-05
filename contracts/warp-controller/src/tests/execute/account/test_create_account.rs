use crate::execute::account::create_account;
use crate::tests::helpers::{instantiate_warp, mock_account_creation_reply};
use crate::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    coin, to_binary, CosmosMsg, ReplyOn, Response, StdError, SubMsg, Uint128, Uint64, WasmMsg,
};

const MOCK_SENDER_ADDRESS: &str = "vlad";

#[test]
fn test_create_account_success() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(MOCK_SENDER_ADDRESS, &vec![coin(100, "uluna")]);

    let _instantiate_res = instantiate_warp(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        Some(info.sender.to_string()),
        Uint64::new(0),
        Uint128::new(0),
        Uint128::new(0),
        Uint128::new(0),
    )
    .unwrap();

    let create_account_res = create_account(deps.as_mut(), env.clone(), info.clone());
    let reply_res =
        mock_account_creation_reply(&mut deps, env.clone(), info.clone(), Uint64::new(0));

    assert_eq!(
        create_account_res.unwrap(),
        Response::new()
            .add_attribute("action", "create_account")
            .add_submessage(SubMsg {
                id: 0,
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    admin: None,
                    code_id: 0,
                    msg: to_binary(&warp_protocol::account::InstantiateMsg {
                        owner: info.sender.to_string(),
                    })
                    .unwrap(),
                    funds: vec![],
                    label: info.sender.to_string(),
                }),
                gas_limit: None,
                reply_on: ReplyOn::Always,
            })
    );

    assert_eq!(
        reply_res.unwrap(),
        Response::new()
            .add_attribute("action", "save_account")
            .add_attribute("owner", info.sender)
            .add_attribute(
                "account_address",
                "terra1vladvladvladvladvladvladvladvladvl1000"
            )
    )
}

#[test]
fn test_create_account_exists() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(MOCK_SENDER_ADDRESS, &vec![coin(100, "uluna")]);

    let _instantiate_res = instantiate_warp(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        Some(info.sender.to_string()),
        Uint64::new(0),
        Uint128::new(0),
        Uint128::new(0),
        Uint128::new(0),
    )
    .unwrap();

    let _create_account_res_first = create_account(deps.as_mut(), env.clone(), info.clone());
    let reply_res_first =
        mock_account_creation_reply(&mut deps, env.clone(), info.clone(), Uint64::new(0));

    let reply_res_first_clone = reply_res_first.unwrap().clone();
    let attr_warp_account_address = reply_res_first_clone
        .attributes
        .iter()
        .find(|attr| attr.key == "account_address")
        .ok_or_else(|| StdError::generic_err("cannot find `account_address` attribute"))
        .unwrap();

    let create_account_res = create_account(deps.as_mut(), env.clone(), info.clone());

    assert_eq!(
        create_account_res.unwrap(),
        Response::new()
            .add_attribute("action", "create_account")
            .add_attribute("owner", info.sender)
            .add_attribute("account_address", attr_warp_account_address.value.as_str())
    );
}

#[test]
fn test_create_account_by_account() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(MOCK_SENDER_ADDRESS, &vec![coin(100, "uluna")]);

    let _instantiate_res = instantiate_warp(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        Some(info.sender.to_string()),
        Uint64::new(0),
        Uint128::new(0),
        Uint128::new(0),
        Uint128::new(0),
    )
    .unwrap();

    let _create_account_res_first = create_account(deps.as_mut(), env.clone(), info.clone());
    let reply_res_first =
        mock_account_creation_reply(&mut deps, env.clone(), info.clone(), Uint64::new(0));

    // Get address of warp account just created and assign it as the sender of next create_account call
    let reply_res_first_clone = reply_res_first.unwrap().clone();
    let attr_warp_account_address = reply_res_first_clone
        .attributes
        .iter()
        .find(|attr| attr.key == "account_address")
        .ok_or_else(|| StdError::generic_err("cannot find `account_address` attribute"))
        .unwrap();
    let info = mock_info(
        attr_warp_account_address.value.as_str(),
        &vec![coin(100, "uluna")],
    );

    let create_account_res = create_account(deps.as_mut(), env.clone(), info.clone());

    assert_eq!(
        create_account_res.unwrap_err(),
        ContractError::AccountCannotCreateAccount {}
    );
}

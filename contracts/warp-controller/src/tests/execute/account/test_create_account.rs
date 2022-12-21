use crate::tests::helpers::{create_warp_account, instantiate_warp};
use crate::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    coin, to_binary, CosmosMsg, ReplyOn, Response, SubMsg, Uint128, Uint64, WasmMsg,
};

#[test]
fn test_create_account_success() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("vlad", &vec![coin(100, "uluna")]);

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

    let (create_account_res, reply_res) =
        create_warp_account(&mut deps, env.clone(), info.clone(), Uint64::new(0));

    assert_eq!(
        create_account_res.unwrap(),
        Response::new()
            .add_attribute("action", "create_account")
            .add_submessage(SubMsg {
                id: 0,
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    admin: None,
                    code_id: 0,
                    msg: to_binary(&warp_protocol::account::account::InstantiateMsg {
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
            .add_attribute("owner", "terra1vladvladvladvladvladvladvladvladvla100")
            .add_attribute(
                "account_address",
                "terra1vladvladvladvladvladvladvladvladvla101"
            )
    )
}

#[test]
fn test_create_account_exists() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("vlad", &vec![coin(100, "uluna")]);

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

    let (_create_account_res_first, _reply_res_first) =
        create_warp_account(&mut deps, env.clone(), info.clone(), Uint64::new(0));
    let (create_account_res, reply_res) =
        create_warp_account(&mut deps, env.clone(), info.clone(), Uint64::new(0));

    assert_eq!(
        create_account_res.unwrap(),
        Response::new()
            .add_attribute("action", "create_account")
            .add_submessage(SubMsg {
                id: 0,
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    admin: None,
                    code_id: 0,
                    msg: to_binary(&warp_protocol::account::account::InstantiateMsg {
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
        reply_res.unwrap_err(),
        ContractError::AccountAlreadyExists {}
    )
}

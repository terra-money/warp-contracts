use cosmwasm_std::{
    coin,
    testing::{mock_dependencies, mock_env, mock_info},
    Api, StdError, Uint128, Uint64,
};
use warp_protocol::controller::account::{Account, AccountResponse, QueryAccountMsg};

use crate::{
    query::account::query_account,
    tests::helpers::{create_warp_account, instantiate_warp},
};

#[test]
fn test_query_account_successful() {
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

    let (_create_account_res, reply_res) =
        create_warp_account(&mut deps, env.clone(), info.clone(), Uint64::new(0));

    // Get address of warp account just created and query it in query_account
    let reply_res_first_clone = reply_res.unwrap().clone();
    let attr_owner = reply_res_first_clone
        .attributes
        .iter()
        .find(|attr| attr.key == "owner")
        .ok_or_else(|| StdError::generic_err("cannot find `owner` attribute"))
        .unwrap();
    let attr_warp_account_address = reply_res_first_clone
        .attributes
        .iter()
        .find(|attr| attr.key == "account_address")
        .ok_or_else(|| StdError::generic_err("cannot find `account_address` attribute"))
        .unwrap();

    let query_account_res = query_account(
        deps.as_ref(),
        env,
        QueryAccountMsg {
            owner: attr_owner.value.clone(),
        },
    );

    assert_eq!(
        query_account_res.unwrap(),
        AccountResponse {
            account: Account {
                owner: deps.api.addr_validate(attr_owner.value.as_str()).unwrap(),
                account: deps
                    .api
                    .addr_validate(attr_warp_account_address.value.as_str())
                    .unwrap(),
            }
        }
    )
}

#[test]
fn test_query_account_does_not_exist() {
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

    let query_account_res = query_account(
        deps.as_ref(),
        env,
        QueryAccountMsg {
            owner: "terra1vladvladvladvladvladvladvladvladvla000".to_string(),
        },
    );

    assert_eq!(
        query_account_res.unwrap_err(),
        StdError::NotFound {
            kind: "warp_protocol::controller::account::Account".to_string()
        }
    )
}

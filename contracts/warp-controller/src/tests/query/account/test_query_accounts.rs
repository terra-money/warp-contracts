use cosmwasm_std::{
    coin,
    testing::{mock_dependencies, mock_env, mock_info},
    Uint128, Uint64,
};
use warp_protocol::controller::account::{AccountsResponse, QueryAccountsMsg};

use crate::{
    query::account::query_accounts,
    state::QUERY_PAGE_SIZE,
    tests::helpers::{create_multiple_warp_accounts, instantiate_warp},
};

#[test]
fn test_query_accounts_successful_when_empty() {
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

    let query_accounts_msgs = vec![
        QueryAccountsMsg {
            start_after: None,
            limit: None,
        },
        QueryAccountsMsg {
            start_after: None,
            limit: Some(10),
        },
        QueryAccountsMsg {
            start_after: Some("terra".to_string()),
            limit: None,
        },
        QueryAccountsMsg {
            start_after: Some("terra".to_string()),
            limit: Some(10),
        },
    ];

    for query_accounts_msg in query_accounts_msgs.iter() {
        let query_accounts_res =
            query_accounts(deps.as_ref(), env.clone(), query_accounts_msg.clone());
        assert_eq!(
            query_accounts_res.unwrap(),
            AccountsResponse { accounts: vec![] }
        );
    }
}

#[test]
fn test_query_accounts_successful_under_50() {
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

    let _create_multiple_account_res = create_multiple_warp_accounts(&mut deps, env.clone(), 30);

    let query_accounts_res = query_accounts(
        deps.as_ref(),
        env.clone(),
        QueryAccountsMsg {
            start_after: None,
            limit: None,
        },
    )
    .unwrap();

    assert_eq!(query_accounts_res.accounts.len(), 30);
    assert_eq!(
        query_accounts_res
            .accounts
            .first()
            .unwrap()
            .owner
            .to_string(),
        "terra1vladvladvladvladvladvladvladvladvl1000"
    );
    assert_eq!(
        query_accounts_res
            .accounts
            .first()
            .unwrap()
            .account
            .to_string(),
        "terra1vladvladvladvladvladvladvladvladvl2000"
    );
    assert_eq!(
        query_accounts_res
            .accounts
            .last()
            .unwrap()
            .owner
            .to_string(),
        "terra1vladvladvladvladvladvladvladvladvl1029"
    );
    assert_eq!(
        query_accounts_res
            .accounts
            .last()
            .unwrap()
            .account
            .to_string(),
        "terra1vladvladvladvladvladvladvladvladvl2029"
    );

    let query_accounts_res = query_accounts(
        deps.as_ref(),
        env.clone(),
        QueryAccountsMsg {
            start_after: Some("terra1vladvladvladvladvladvladvladvladvl1009".to_string()),
            limit: Some(10),
        },
    )
    .unwrap();

    assert_eq!(query_accounts_res.accounts.len(), 10);
    assert_eq!(
        query_accounts_res
            .accounts
            .first()
            .unwrap()
            .owner
            .to_string(),
        "terra1vladvladvladvladvladvladvladvladvl1010"
    );
    assert_eq!(
        query_accounts_res
            .accounts
            .last()
            .unwrap()
            .owner
            .to_string(),
        "terra1vladvladvladvladvladvladvladvladvl1019"
    )
}

#[test]
fn test_query_accounts_successful_over_50_paginated() {
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

    let _create_multiple_account_res = create_multiple_warp_accounts(&mut deps, env.clone(), 150);

    let query_accounts_res = query_accounts(
        deps.as_ref(),
        env.clone(),
        QueryAccountsMsg {
            start_after: Some("terra1vladvladvladvladvladvladvladvladvl1059".to_string()),
            limit: None,
        },
    )
    .unwrap();

    assert_eq!(
        query_accounts_res.accounts.len(),
        usize::try_from(QUERY_PAGE_SIZE).unwrap()
    );
    assert_eq!(
        query_accounts_res
            .accounts
            .first()
            .unwrap()
            .owner
            .to_string(),
        "terra1vladvladvladvladvladvladvladvladvl1060"
    );
    assert_eq!(
        query_accounts_res
            .accounts
            .last()
            .unwrap()
            .owner
            .to_string(),
        "terra1vladvladvladvladvladvladvladvladvl1109"
    )
}

#[test]
fn test_query_accounts_limit_over_50() {
    //should fail out here
}

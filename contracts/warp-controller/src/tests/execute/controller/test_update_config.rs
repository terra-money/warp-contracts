use crate::execute::controller::update_config;
use crate::tests::helpers::instantiate_warp;
use crate::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, Response, Uint128, Uint64};
use warp_protocol::controller::controller::UpdateConfigMsg;

#[test]
fn test_modify_config_success() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("vlad", &vec![coin(100, "uluna")]);

    let instantiate_res = instantiate_warp(
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

    let modify_config_msg = UpdateConfigMsg {
        owner: Some(info.sender.to_string()),
        minimum_reward: Some(Uint128::new(1)),
        creation_fee_percentage: Some(Uint128::new(2)),
        cancellation_fee_percentage: Some(Uint128::new(3)),
    };

    let modify_config_res =
        update_config(deps.as_mut(), env.clone(), info.clone(), modify_config_msg).unwrap();

    assert_eq!(
        modify_config_res,
        Response::new()
            .add_attribute("action", "update_config")
            .add_attribute("config_owner", info.sender.to_string())
            .add_attribute("config_minimum_reward", Uint128::new(1))
            .add_attribute("config_creation_fee_percentage", Uint128::new(2),)
            .add_attribute("config_cancellation_fee_percentage", Uint128::new(3),)
    )
}

#[test]
fn test_modify_config_unauthorized() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("vlad", &vec![coin(100, "uluna")]);

    let instantiate_res = instantiate_warp(
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

    let modify_config_msg = UpdateConfigMsg {
        owner: Some(info.sender.to_string()),
        minimum_reward: Some(Uint128::new(1)),
        creation_fee_percentage: Some(Uint128::new(2)),
        cancellation_fee_percentage: Some(Uint128::new(3)),
    };

    let info = mock_info("vlad2", &vec![coin(100, "uluna")]);

    let modify_config_res =
        update_config(deps.as_mut(), env.clone(), info.clone(), modify_config_msg).unwrap_err();

    assert_eq!(modify_config_res, ContractError::Unauthorized {})
}

#[test]
fn test_modify_config_bad_percentages() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("vlad", &vec![coin(100, "uluna")]);

    let instantiate_res = instantiate_warp(
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

    let modify_config_msg = UpdateConfigMsg {
        owner: Some(info.sender.to_string()),
        minimum_reward: Some(Uint128::new(1)),
        creation_fee_percentage: Some(Uint128::new(101)),
        cancellation_fee_percentage: Some(Uint128::new(3)),
    };

    let modify_config_res =
        update_config(deps.as_mut(), env.clone(), info.clone(), modify_config_msg).unwrap_err();

    assert_eq!(modify_config_res, ContractError::CreationFeeTooHigh {});

    let modify_config_msg = UpdateConfigMsg {
        owner: Some(info.sender.to_string()),
        minimum_reward: Some(Uint128::new(1)),
        creation_fee_percentage: Some(Uint128::new(100)),
        cancellation_fee_percentage: Some(Uint128::new(101)),
    };

    let modify_config_res =
        update_config(deps.as_mut(), env.clone(), info.clone(), modify_config_msg).unwrap_err();

    assert_eq!(modify_config_res, ContractError::CancellationFeeTooHigh {})
}

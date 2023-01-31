use crate::execute::controller::update_config;
use crate::tests::helpers::instantiate_warp;
use crate::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, Response, Uint128, Uint64};
use warp_protocol::controller::UpdateConfigMsg;

#[test]
fn test_modify_config_success() {
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
        Uint64::new(0),
        Uint64::new(0),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
    )
    .unwrap();

    let modify_config_msg = UpdateConfigMsg {
        owner: Some(info.sender.to_string()),
        minimum_reward: Some(Uint128::new(7)),
        creation_fee_percentage: Some(Uint64::new(2)),
        cancellation_fee_percentage: Some(Uint64::new(3)),
        template_fee: Some(Uint128::new(4)),
        t_max: Some(Uint64::new(6)),
        t_min: Some(Uint64::new(5)),
        a_max: Some(Uint128::new(8)),
        a_min: Some(Uint128::new(7)),
        q_max: Some(Uint64::new(9)),
    };

    let modify_config_res =
        update_config(deps.as_mut(), env.clone(), info.clone(), modify_config_msg).unwrap();

    assert_eq!(
        modify_config_res,
        Response::new()
            .add_attribute("action", "update_config")
            .add_attribute("config_owner", info.sender.to_string())
            .add_attribute("config_minimum_reward", Uint128::new(7))
            .add_attribute("config_creation_fee_percentage", Uint64::new(2),)
            .add_attribute("config_cancellation_fee_percentage", Uint64::new(3))
            .add_attribute("config_template_fee", Uint128::new(4))
            .add_attribute("config_a_max", Uint128::new(8))
            .add_attribute("config_a_min", Uint128::new(7))
            .add_attribute("config_t_max", Uint64::new(6))
            .add_attribute("config_t_min", Uint64::new(5))
            .add_attribute("config_q_max", Uint64::new(9))
    )
}

#[test]
fn test_modify_config_unauthorized() {
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
        Uint64::new(0),
        Uint64::new(0),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
    )
    .unwrap();

    let modify_config_msg = UpdateConfigMsg {
        owner: Some(info.sender.to_string()),
        minimum_reward: Some(Uint128::new(1)),
        creation_fee_percentage: Some(Uint64::new(2)),
        cancellation_fee_percentage: Some(Uint64::new(3)),
        template_fee: Some(Uint128::new(4)),
        t_max: Some(Uint64::new(5)),
        t_min: Some(Uint64::new(6)),
        a_max: Some(Uint128::new(7)),
        a_min: Some(Uint128::new(8)),
        q_max: Some(Uint64::new(9)),
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

    let _instantiate_res = instantiate_warp(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        Some(info.sender.to_string()),
        Uint64::new(0),
        Uint128::new(0),
        Uint64::new(0),
        Uint64::new(0),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
    )
    .unwrap();

    let modify_config_msg = UpdateConfigMsg {
        owner: Some(info.sender.to_string()),
        minimum_reward: Some(Uint128::new(7)),
        creation_fee_percentage: Some(Uint64::new(101)),
        cancellation_fee_percentage: Some(Uint64::new(3)),
        template_fee: Some(Uint128::new(4)),
        t_max: Some(Uint64::new(6)),
        t_min: Some(Uint64::new(5)),
        a_max: Some(Uint128::new(8)),
        a_min: Some(Uint128::new(7)),
        q_max: Some(Uint64::new(9)),
    };

    let modify_config_res =
        update_config(deps.as_mut(), env.clone(), info.clone(), modify_config_msg).unwrap_err();

    assert_eq!(modify_config_res, ContractError::CreationFeeTooHigh {});

    let modify_config_msg = UpdateConfigMsg {
        owner: Some(info.sender.to_string()),
        minimum_reward: Some(Uint128::new(100)),
        creation_fee_percentage: Some(Uint64::new(2)),
        cancellation_fee_percentage: Some(Uint64::new(101)),
        template_fee: Some(Uint128::new(4)),
        t_max: Some(Uint64::new(6)),
        t_min: Some(Uint64::new(5)),
        a_max: Some(Uint128::new(8)),
        a_min: Some(Uint128::new(7)),
        q_max: Some(Uint64::new(9)),
    };

    let modify_config_res =
        update_config(deps.as_mut(), env.clone(), info.clone(), modify_config_msg).unwrap_err();

    assert_eq!(modify_config_res, ContractError::CancellationFeeTooHigh {})
}

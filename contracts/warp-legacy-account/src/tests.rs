use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    to_binary, BankMsg, Coin, CosmosMsg, DistributionMsg, GovMsg, IbcMsg, IbcTimeout,
    IbcTimeoutBlock, Response, StakingMsg, Uint128, Uint64, VoteOption, WasmMsg,
};

use crate::contract::{execute, instantiate};
use crate::ContractError;
use legacy_account::{ExecuteMsg, GenericMsg, InstantiateMsg};

#[test]
fn test_execute_controller() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("vlad_controller", &[]);

    let _instantiate_res = instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            owner: "vlad".to_string(),
            job_id: Uint64::zero(),
            job_account_tracker_addr: "vlad".to_string(),
            native_funds: vec![],
            cw_funds: vec![],
            msgs: vec![],
        },
    );

    let execute_msg = ExecuteMsg::Generic(GenericMsg {
        msgs: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "contract".to_string(),
                msg: to_binary("test").unwrap(),
                funds: vec![Coin {
                    denom: "coin".to_string(),
                    amount: Uint128::new(100),
                }],
            }),
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "vlad2".to_string(),
                amount: vec![Coin {
                    denom: "coin".to_string(),
                    amount: Uint128::new(100),
                }],
            }),
            CosmosMsg::Gov(GovMsg::Vote {
                proposal_id: 0,
                vote: VoteOption::Yes,
            }),
            CosmosMsg::Staking(StakingMsg::Delegate {
                validator: "vladidator".to_string(),
                amount: Coin {
                    denom: "coin".to_string(),
                    amount: Uint128::new(100),
                },
            }),
            CosmosMsg::Distribution(DistributionMsg::SetWithdrawAddress {
                address: "vladdress".to_string(),
            }),
            CosmosMsg::Ibc(IbcMsg::Transfer {
                channel_id: "channel_vlad".to_string(),
                to_address: "vlad3".to_string(),
                amount: Coin {
                    denom: "coin".to_string(),
                    amount: Uint128::new(100),
                },
                timeout: IbcTimeout::with_block(IbcTimeoutBlock {
                    revision: 0,
                    height: 0,
                }),
            }),
            CosmosMsg::Stargate {
                type_url: "utl".to_string(),
                value: Default::default(),
            },
        ],
    });

    let execute_res = execute(deps.as_mut(), env, info, execute_msg).unwrap();

    assert_eq!(
        execute_res,
        Response::new()
            .add_attribute("action", "generic")
            .add_messages(vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: "contract".to_string(),
                    msg: to_binary("test").unwrap(),
                    funds: vec![Coin {
                        denom: "coin".to_string(),
                        amount: Uint128::new(100)
                    }],
                }),
                CosmosMsg::Bank(BankMsg::Send {
                    to_address: "vlad2".to_string(),
                    amount: vec![Coin {
                        denom: "coin".to_string(),
                        amount: Uint128::new(100)
                    }]
                }),
                CosmosMsg::Gov(GovMsg::Vote {
                    proposal_id: 0,
                    vote: VoteOption::Yes
                }),
                CosmosMsg::Staking(StakingMsg::Delegate {
                    validator: "vladidator".to_string(),
                    amount: Coin {
                        denom: "coin".to_string(),
                        amount: Uint128::new(100)
                    },
                }),
                CosmosMsg::Distribution(DistributionMsg::SetWithdrawAddress {
                    address: "vladdress".to_string(),
                }),
                CosmosMsg::Ibc(IbcMsg::Transfer {
                    channel_id: "channel_vlad".to_string(),
                    to_address: "vlad3".to_string(),
                    amount: Coin {
                        denom: "coin".to_string(),
                        amount: Uint128::new(100)
                    },
                    timeout: IbcTimeout::with_block(IbcTimeoutBlock {
                        revision: 0,
                        height: 0
                    }),
                }),
                CosmosMsg::Stargate {
                    type_url: "utl".to_string(),
                    value: Default::default()
                }
            ])
    )
}

#[test]
fn test_execute_owner() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("vlad_controller", &[]);

    let _instantiate_res = instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            owner: "vlad".to_string(),
            job_id: Uint64::zero(),
            job_account_tracker_addr: "vlad".to_string(),
            native_funds: vec![],
            cw_funds: vec![],
            msgs: vec![],
        },
    );

    let execute_msg = ExecuteMsg::Generic(GenericMsg {
        msgs: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "contract".to_string(),
                msg: to_binary("test").unwrap(),
                funds: vec![Coin {
                    denom: "coin".to_string(),
                    amount: Uint128::new(100),
                }],
            }),
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "vlad2".to_string(),
                amount: vec![Coin {
                    denom: "coin".to_string(),
                    amount: Uint128::new(100),
                }],
            }),
            CosmosMsg::Gov(GovMsg::Vote {
                proposal_id: 0,
                vote: VoteOption::Yes,
            }),
            CosmosMsg::Staking(StakingMsg::Delegate {
                validator: "vladidator".to_string(),
                amount: Coin {
                    denom: "coin".to_string(),
                    amount: Uint128::new(100),
                },
            }),
            CosmosMsg::Distribution(DistributionMsg::SetWithdrawAddress {
                address: "vladdress".to_string(),
            }),
            CosmosMsg::Ibc(IbcMsg::Transfer {
                channel_id: "channel_vlad".to_string(),
                to_address: "vlad3".to_string(),
                amount: Coin {
                    denom: "coin".to_string(),
                    amount: Uint128::new(100),
                },
                timeout: IbcTimeout::with_block(IbcTimeoutBlock {
                    revision: 0,
                    height: 0,
                }),
            }),
            CosmosMsg::Stargate {
                type_url: "utl".to_string(),
                value: Default::default(),
            },
        ],
    });

    let info2 = mock_info("vlad", &[]);

    let execute_res = execute(deps.as_mut(), env, info2, execute_msg).unwrap();

    assert_eq!(
        execute_res,
        Response::new()
            .add_attribute("action", "generic")
            .add_messages(vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: "contract".to_string(),
                    msg: to_binary("test").unwrap(),
                    funds: vec![Coin {
                        denom: "coin".to_string(),
                        amount: Uint128::new(100)
                    }],
                }),
                CosmosMsg::Bank(BankMsg::Send {
                    to_address: "vlad2".to_string(),
                    amount: vec![Coin {
                        denom: "coin".to_string(),
                        amount: Uint128::new(100)
                    }]
                }),
                CosmosMsg::Gov(GovMsg::Vote {
                    proposal_id: 0,
                    vote: VoteOption::Yes
                }),
                CosmosMsg::Staking(StakingMsg::Delegate {
                    validator: "vladidator".to_string(),
                    amount: Coin {
                        denom: "coin".to_string(),
                        amount: Uint128::new(100)
                    },
                }),
                CosmosMsg::Distribution(DistributionMsg::SetWithdrawAddress {
                    address: "vladdress".to_string(),
                }),
                CosmosMsg::Ibc(IbcMsg::Transfer {
                    channel_id: "channel_vlad".to_string(),
                    to_address: "vlad3".to_string(),
                    amount: Coin {
                        denom: "coin".to_string(),
                        amount: Uint128::new(100)
                    },
                    timeout: IbcTimeout::with_block(IbcTimeoutBlock {
                        revision: 0,
                        height: 0
                    }),
                }),
                CosmosMsg::Stargate {
                    type_url: "utl".to_string(),
                    value: Default::default()
                }
            ])
    )
}

#[test]
fn test_execute_unauth() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("vlad_controller", &[]);

    let _instantiate_res = instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            owner: "vlad".to_string(),
            job_id: Uint64::zero(),
            job_account_tracker_addr: "vlad".to_string(),
            native_funds: vec![],
            cw_funds: vec![],
            msgs: vec![],
        },
    );

    let execute_msg = ExecuteMsg::Generic(GenericMsg {
        msgs: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "contract".to_string(),
                msg: to_binary("test").unwrap(),
                funds: vec![Coin {
                    denom: "coin".to_string(),
                    amount: Uint128::new(100),
                }],
            }),
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "vlad2".to_string(),
                amount: vec![Coin {
                    denom: "coin".to_string(),
                    amount: Uint128::new(100),
                }],
            }),
            CosmosMsg::Gov(GovMsg::Vote {
                proposal_id: 0,
                vote: VoteOption::Yes,
            }),
            CosmosMsg::Staking(StakingMsg::Delegate {
                validator: "vladidator".to_string(),
                amount: Coin {
                    denom: "coin".to_string(),
                    amount: Uint128::new(100),
                },
            }),
            CosmosMsg::Distribution(DistributionMsg::SetWithdrawAddress {
                address: "vladdress".to_string(),
            }),
            CosmosMsg::Ibc(IbcMsg::Transfer {
                channel_id: "channel_vlad".to_string(),
                to_address: "vlad3".to_string(),
                amount: Coin {
                    denom: "coin".to_string(),
                    amount: Uint128::new(100),
                },
                timeout: IbcTimeout::with_block(IbcTimeoutBlock {
                    revision: 0,
                    height: 0,
                }),
            }),
            CosmosMsg::Stargate {
                type_url: "utl".to_string(),
                value: Default::default(),
            },
        ],
    });

    let info2 = mock_info("vlad2", &[]);

    let execute_res = execute(deps.as_mut(), env, info2, execute_msg).unwrap_err();

    assert_eq!(execute_res, ContractError::Unauthorized {})
}

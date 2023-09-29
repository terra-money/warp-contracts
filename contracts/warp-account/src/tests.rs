use crate::contract::{execute, instantiate, query};
use crate::ContractError;
use account::{
    Config, ConfigResponse, ExecuteMsg, FirstFreeSubAccountResponse, FreeSubAccountsResponse,
    GenericMsg, InstantiateMsg, OccupiedSubAccountsResponse, QueryConfigMsg,
    QueryFirstFreeSubAccountMsg, QueryFreeSubAccountsMsg, QueryMsg, QueryOccupiedSubAccountsMsg,
};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    to_binary, Api, BankMsg, Coin, CosmosMsg, DistributionMsg, GovMsg, IbcMsg, IbcTimeout,
    IbcTimeoutBlock, Response, StakingMsg, Uint128, VoteOption, WasmMsg,
};

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
            funds: None,
            msgs: None,
            is_sub_account: Some(false),
            main_account_addr: None,
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
            funds: None,
            msgs: None,
            is_sub_account: Some(false),
            main_account_addr: None,
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
            funds: None,
            msgs: None,
            is_sub_account: Some(false),
            main_account_addr: None,
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

#[test]
fn test_manage_sub_account() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let warp_controller = "vlad_controller";
    let owner = "vlad";
    let info = mock_info(warp_controller, &[]);

    let instantiate_msg = InstantiateMsg {
        owner: owner.to_string(),
        funds: None,
        msgs: None,
        is_sub_account: Some(false),
        main_account_addr: None,
    };
    let instantiate_main_account_res = instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        instantiate_msg.clone(),
    )
    .unwrap();

    let main_account_addr = instantiate_main_account_res
        .attributes
        .iter()
        .find(|attr| attr.key == "contract_addr")
        .unwrap()
        .value
        .clone();

    assert_eq!(
        instantiate_main_account_res,
        Response::new()
            .add_attribute("action", "instantiate")
            .add_attribute("contract_addr", main_account_addr.clone())
            .add_attribute("is_sub_account", "false")
            .add_attribute("main_account_addr", main_account_addr.clone())
            .add_attribute("owner", owner)
            .add_attribute("funds", serde_json_wasm::to_string(&info.funds).unwrap())
            .add_attribute(
                "cw_funds",
                serde_json_wasm::to_string(&instantiate_msg.clone().funds).unwrap()
            )
            .add_attribute(
                "account_msgs",
                serde_json_wasm::to_string(&instantiate_msg.clone().msgs).unwrap()
            )
    );

    let query_config_msg = QueryMsg::QueryConfig(QueryConfigMsg {});
    let query_res = query(deps.as_ref(), env.clone(), query_config_msg).unwrap();
    assert_eq!(
        query_res,
        to_binary(&ConfigResponse {
            config: Config {
                owner: deps.api.addr_validate(owner).unwrap(),
                warp_addr: deps.api.addr_validate(warp_controller).unwrap(),
                is_sub_account: false,
                main_account_addr: deps.api.addr_validate(main_account_addr.as_str()).unwrap(),
            }
        })
        .unwrap()
    );

    let query_first_free_sub_account_msg =
        QueryMsg::QueryFirstFreeSubAccount(QueryFirstFreeSubAccountMsg {});
    let query_res = query(deps.as_ref(), env.clone(), query_first_free_sub_account_msg).unwrap();
    assert_eq!(
        query_res,
        to_binary(&FirstFreeSubAccountResponse { sub_account: None }).unwrap()
    );

    let query_free_sub_accounts_msg = QueryMsg::QueryFreeSubAccounts(QueryFreeSubAccountsMsg {
        start_after: None,
        limit: None,
    });
    let query_res = query(deps.as_ref(), env.clone(), query_free_sub_accounts_msg).unwrap();
    assert_eq!(
        query_res,
        to_binary(&FreeSubAccountsResponse {
            sub_accounts: vec![],
            total_count: 0
        })
        .unwrap()
    );

    let query_occupied_sub_accounts_msg =
        QueryMsg::QueryOccupiedSubAccounts(QueryOccupiedSubAccountsMsg {
            start_after: None,
            limit: None,
        });
    let query_res = query(deps.as_ref(), env.clone(), query_occupied_sub_accounts_msg).unwrap();
    assert_eq!(
        query_res,
        to_binary(&OccupiedSubAccountsResponse {
            sub_accounts: vec![],
            total_count: 0
        })
        .unwrap()
    );

    let _instantiate_sub_account_res = instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            owner: "vlad".to_string(),
            funds: None,
            msgs: None,
            is_sub_account: Some(true),
            main_account_addr: None,
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

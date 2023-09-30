#[cfg(test)]
mod tests {
    use account::{
        Config, ConfigResponse, ExecuteMsg, FirstFreeSubAccountResponse, FreeSubAccountMsg,
        FreeSubAccountsResponse, InstantiateMsg, OccupiedSubAccountsResponse, OccupySubAccountMsg,
        QueryConfigMsg, QueryFirstFreeSubAccountMsg, QueryFreeSubAccountsMsg, QueryMsg,
        QueryOccupiedSubAccountsMsg, SubAccount,
    };
    use anyhow::Result as AnyResult;
    use cosmwasm_std::{Addr, Coin, Empty, Uint128, Uint64};
    use cw_multi_test::{App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor};

    use crate::{
        contract::{execute, instantiate, query},
        ContractError,
    };

    const USER_1: &str = "terra1";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER_1),
                    vec![Coin {
                        denom: "uluna".to_string(),
                        // 1_000_000_000 uLuna i.e. 1k LUNA since 1 LUNA = 1_000_000 uLuna
                        amount: Uint128::new(1_000_000_000),
                    }],
                )
                .unwrap();
        })
    }

    fn contract_warp_account() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    fn init_warp_account(
        app: &mut App,
        warp_account_contract_code_id: u64,
        is_sub_account: Option<bool>,
        main_account_addr: Option<String>,
    ) -> Addr {
        app.instantiate_contract(
            warp_account_contract_code_id,
            Addr::unchecked(USER_1),
            &InstantiateMsg {
                owner: USER_1.to_string(),
                msgs: None,
                funds: None,
                is_sub_account,
                main_account_addr,
            },
            &[],
            "warp_main_account",
            None,
        )
        .unwrap()
    }

    fn assert_err(res: AnyResult<AppResponse>, err: ContractError) {
        match res {
            Ok(_) => panic!("Result was not an error"),
            Err(generic_err) => {
                let contract_err: ContractError = generic_err.downcast().unwrap();
                assert_eq!(contract_err, err);
            }
        }
    }

    #[test]
    fn warp_account_contract_multi_test_sub_account_management() {
        let mut app = mock_app();
        let warp_account_contract_code_id = app.store_code(contract_warp_account());

        // Instantiate main account
        let warp_main_account_contract_addr =
            init_warp_account(&mut app, warp_account_contract_code_id, Some(false), None);
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_main_account_contract_addr.clone(),
                &QueryMsg::QueryConfig(QueryConfigMsg {})
            ),
            Ok(ConfigResponse {
                config: Config {
                    owner: Addr::unchecked(USER_1),
                    warp_addr: Addr::unchecked(USER_1),
                    is_sub_account: false,
                    main_account_addr: Addr::unchecked(warp_main_account_contract_addr.clone())
                }
            })
        );
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_main_account_contract_addr.clone(),
                &QueryMsg::QueryFirstFreeSubAccount(QueryFirstFreeSubAccountMsg {})
            ),
            Ok(FirstFreeSubAccountResponse { sub_account: None })
        );
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_main_account_contract_addr.clone(),
                &QueryMsg::QueryFreeSubAccounts(QueryFreeSubAccountsMsg {
                    start_after: None,
                    limit: None
                })
            ),
            Ok(FreeSubAccountsResponse {
                sub_accounts: vec![],
                total_count: 0
            })
        );
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_main_account_contract_addr.clone(),
                &QueryMsg::QueryOccupiedSubAccounts(QueryOccupiedSubAccountsMsg {
                    start_after: None,
                    limit: None
                })
            ),
            Ok(OccupiedSubAccountsResponse {
                sub_accounts: vec![],
                total_count: 0
            })
        );

        // Instantiate first sub account
        let warp_sub_account_1_contract_addr = init_warp_account(
            &mut app,
            warp_account_contract_code_id,
            Some(true),
            Some(warp_main_account_contract_addr.to_string()),
        );
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_sub_account_1_contract_addr.clone(),
                &QueryMsg::QueryConfig(QueryConfigMsg {})
            ),
            Ok(ConfigResponse {
                config: Config {
                    owner: Addr::unchecked(USER_1),
                    warp_addr: Addr::unchecked(USER_1),
                    is_sub_account: true,
                    main_account_addr: Addr::unchecked(warp_main_account_contract_addr.clone())
                }
            })
        );
        // Mark first sub account as free
        let _ = app.execute_contract(
            Addr::unchecked(USER_1),
            warp_main_account_contract_addr.clone(),
            &ExecuteMsg::FreeSubAccount(FreeSubAccountMsg {
                sub_account_addr: warp_sub_account_1_contract_addr.to_string(),
            }),
            &[],
        );
        // Cannot free sub account twice
        assert_err(
            app.execute_contract(
                Addr::unchecked(USER_1),
                warp_main_account_contract_addr.clone(),
                &ExecuteMsg::FreeSubAccount(FreeSubAccountMsg {
                    sub_account_addr: warp_sub_account_1_contract_addr.to_string(),
                }),
                &[],
            ),
            ContractError::SubAccountAlreadyFreeError {},
        );

        // Instantiate second sub account
        let warp_sub_account_2_contract_addr = init_warp_account(
            &mut app,
            warp_account_contract_code_id,
            Some(true),
            Some(warp_main_account_contract_addr.to_string()),
        );
        // Mark second sub account as free
        let _ = app.execute_contract(
            Addr::unchecked(USER_1),
            warp_main_account_contract_addr.clone(),
            &ExecuteMsg::FreeSubAccount(FreeSubAccountMsg {
                sub_account_addr: warp_sub_account_2_contract_addr.to_string(),
            }),
            &[],
        );

        // Instantiate third sub account
        let warp_sub_account_3_contract_addr = init_warp_account(
            &mut app,
            warp_account_contract_code_id,
            Some(true),
            Some(warp_main_account_contract_addr.to_string()),
        );
        // Mark third sub account as free
        let _ = app.execute_contract(
            Addr::unchecked(USER_1),
            warp_main_account_contract_addr.clone(),
            &ExecuteMsg::FreeSubAccount(FreeSubAccountMsg {
                sub_account_addr: warp_sub_account_3_contract_addr.to_string(),
            }),
            &[],
        );

        // Query first free sub account
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_main_account_contract_addr.clone(),
                &QueryMsg::QueryFirstFreeSubAccount(QueryFirstFreeSubAccountMsg {})
            ),
            Ok(FirstFreeSubAccountResponse {
                sub_account: Some(SubAccount {
                    addr: warp_sub_account_1_contract_addr.to_string(),
                    in_use_by_job_id: None
                })
            })
        );

        // Query free sub accounts
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_main_account_contract_addr.clone(),
                &QueryMsg::QueryFreeSubAccounts(QueryFreeSubAccountsMsg {
                    start_after: None,
                    limit: None
                })
            ),
            Ok(FreeSubAccountsResponse {
                sub_accounts: vec![
                    SubAccount {
                        addr: warp_sub_account_3_contract_addr.to_string(),
                        in_use_by_job_id: None
                    },
                    SubAccount {
                        addr: warp_sub_account_2_contract_addr.to_string(),
                        in_use_by_job_id: None
                    },
                    SubAccount {
                        addr: warp_sub_account_1_contract_addr.to_string(),
                        in_use_by_job_id: None
                    }
                ],
                total_count: 3
            })
        );

        // Query occupied sub accounts
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_main_account_contract_addr.clone(),
                &QueryMsg::QueryOccupiedSubAccounts(QueryOccupiedSubAccountsMsg {
                    start_after: None,
                    limit: None
                })
            ),
            Ok(OccupiedSubAccountsResponse {
                sub_accounts: vec![],
                total_count: 0
            })
        );

        // Occupy second sub account
        let _ = app.execute_contract(
            Addr::unchecked(USER_1),
            warp_main_account_contract_addr.clone(),
            &ExecuteMsg::OccupySubAccount(OccupySubAccountMsg {
                sub_account_addr: warp_sub_account_2_contract_addr.to_string(),
                job_id: Uint64::from(1 as u8),
            }),
            &[],
        );
        // Cannot occupy sub account twice
        assert_err(
            app.execute_contract(
                Addr::unchecked(USER_1),
                warp_main_account_contract_addr.clone(),
                &ExecuteMsg::OccupySubAccount(OccupySubAccountMsg {
                    sub_account_addr: warp_sub_account_2_contract_addr.to_string(),
                    job_id: Uint64::from(1 as u8),
                }),
                &[],
            ),
            ContractError::SubAccountAlreadyOccupiedError {},
        );

        // Query free sub accounts
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_main_account_contract_addr.clone(),
                &QueryMsg::QueryFreeSubAccounts(QueryFreeSubAccountsMsg {
                    start_after: None,
                    limit: None
                })
            ),
            Ok(FreeSubAccountsResponse {
                sub_accounts: vec![
                    SubAccount {
                        addr: warp_sub_account_3_contract_addr.to_string(),
                        in_use_by_job_id: None
                    },
                    SubAccount {
                        addr: warp_sub_account_1_contract_addr.to_string(),
                        in_use_by_job_id: None
                    }
                ],
                total_count: 2
            })
        );

        // Query occupied sub accounts
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_main_account_contract_addr.clone(),
                &QueryMsg::QueryOccupiedSubAccounts(QueryOccupiedSubAccountsMsg {
                    start_after: None,
                    limit: None
                })
            ),
            Ok(OccupiedSubAccountsResponse {
                sub_accounts: vec![SubAccount {
                    addr: warp_sub_account_2_contract_addr.to_string(),
                    in_use_by_job_id: Some(Uint64::from(1 as u8))
                }],
                total_count: 1
            })
        );

        // Free second sub account
        let _ = app.execute_contract(
            Addr::unchecked(USER_1),
            warp_main_account_contract_addr.clone(),
            &ExecuteMsg::FreeSubAccount(FreeSubAccountMsg {
                sub_account_addr: warp_sub_account_2_contract_addr.to_string(),
            }),
            &[],
        );

        // Query free sub accounts
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_main_account_contract_addr.clone(),
                &QueryMsg::QueryFreeSubAccounts(QueryFreeSubAccountsMsg {
                    start_after: None,
                    limit: None
                })
            ),
            Ok(FreeSubAccountsResponse {
                sub_accounts: vec![
                    SubAccount {
                        addr: warp_sub_account_3_contract_addr.to_string(),
                        in_use_by_job_id: None
                    },
                    SubAccount {
                        addr: warp_sub_account_2_contract_addr.to_string(),
                        in_use_by_job_id: None
                    },
                    SubAccount {
                        addr: warp_sub_account_1_contract_addr.to_string(),
                        in_use_by_job_id: None
                    }
                ],
                total_count: 3
            })
        );

        // Query occupied sub accounts
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_main_account_contract_addr.clone(),
                &QueryMsg::QueryOccupiedSubAccounts(QueryOccupiedSubAccountsMsg {
                    start_after: None,
                    limit: None
                })
            ),
            Ok(OccupiedSubAccountsResponse {
                sub_accounts: vec![],
                total_count: 0
            })
        );
    }
}

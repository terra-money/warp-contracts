#[cfg(test)]
mod tests {
    use anyhow::Result as AnyResult;
    use cosmwasm_std::{Addr, Coin, Empty, Uint128, Uint64};
    use cw_multi_test::{App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor};
    use job_account_tracker::{
        Account, AccountsResponse, Config, ConfigResponse, ExecuteMsg, FirstFreeAccountResponse,
        FreeAccountMsg, InstantiateMsg, OccupyAccountMsg, QueryConfigMsg, QueryFirstFreeAccountMsg,
        QueryFreeAccountsMsg, QueryMsg, QueryOccupiedAccountsMsg,
    };

    use crate::{
        contract::{execute, instantiate, query},
        ContractError,
    };

    const DUMMY_WARP_CONTROLLER_ADDR: &str = "terra1";
    const USER_1: &str = "terra2";
    const DUMMY_WARP_ACCOUNT_1_ADDR: &str = "terra3";
    const DUMMY_WARP_ACCOUNT_2_ADDR: &str = "terra4";
    const DUMMY_WARP_ACCOUNT_3_ADDR: &str = "terra5";
    const DUMMY_JOB_1_ID: Uint64 = Uint64::zero();
    const DUMMY_JOB_2_ID: Uint64 = Uint64::one();

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

    fn contract_warp_job_account_tracker() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    fn init_warp_job_account_tracker(
        app: &mut App,
        warp_job_account_tracker_contract_code_id: u64,
    ) -> Addr {
        app.instantiate_contract(
            warp_job_account_tracker_contract_code_id,
            Addr::unchecked(DUMMY_WARP_CONTROLLER_ADDR),
            &InstantiateMsg {
                owner: USER_1.to_string(),
            },
            &[],
            "warp_job_account_tracker",
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
    fn warp_job_account_tracker_contract_multi_test_account_management() {
        let mut app = mock_app();
        let warp_job_account_tracker_contract_code_id =
            app.store_code(contract_warp_job_account_tracker());

        // Instantiate account
        let warp_job_account_tracker_contract_addr =
            init_warp_job_account_tracker(&mut app, warp_job_account_tracker_contract_code_id);
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_job_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryConfig(QueryConfigMsg {})
            ),
            Ok(ConfigResponse {
                config: Config {
                    owner: Addr::unchecked(USER_1),
                    creator_addr: Addr::unchecked(DUMMY_WARP_CONTROLLER_ADDR),
                }
            })
        );
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_job_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryFirstFreeAccount(QueryFirstFreeAccountMsg {})
            ),
            Ok(FirstFreeAccountResponse { account: None })
        );
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_job_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryFreeAccounts(QueryFreeAccountsMsg {
                    start_after: None,
                    limit: None
                })
            ),
            Ok(AccountsResponse {
                accounts: vec![],
                total_count: 0
            })
        );
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_job_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryOccupiedAccounts(QueryOccupiedAccountsMsg {
                    start_after: None,
                    limit: None
                })
            ),
            Ok(AccountsResponse {
                accounts: vec![],
                total_count: 0
            })
        );

        // Mark first account as free
        let _ = app.execute_contract(
            Addr::unchecked(USER_1),
            warp_job_account_tracker_contract_addr.clone(),
            &ExecuteMsg::FreeAccount(FreeAccountMsg {
                account_addr: DUMMY_WARP_ACCOUNT_1_ADDR.to_string(),
            }),
            &[],
        );

        // Cannot free account twice
        assert_err(
            app.execute_contract(
                Addr::unchecked(USER_1),
                warp_job_account_tracker_contract_addr.clone(),
                &ExecuteMsg::FreeAccount(FreeAccountMsg {
                    account_addr: DUMMY_WARP_ACCOUNT_1_ADDR.to_string(),
                }),
                &[],
            ),
            ContractError::AccountAlreadyFreeError {},
        );

        // Mark second account as free
        let _ = app.execute_contract(
            Addr::unchecked(USER_1),
            warp_job_account_tracker_contract_addr.clone(),
            &ExecuteMsg::FreeAccount(FreeAccountMsg {
                account_addr: DUMMY_WARP_ACCOUNT_2_ADDR.to_string(),
            }),
            &[],
        );

        // Mark third account as free
        let _ = app.execute_contract(
            Addr::unchecked(USER_1),
            warp_job_account_tracker_contract_addr.clone(),
            &ExecuteMsg::FreeAccount(FreeAccountMsg {
                account_addr: DUMMY_WARP_ACCOUNT_3_ADDR.to_string(),
            }),
            &[],
        );

        // Query first free account
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_job_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryFirstFreeAccount(QueryFirstFreeAccountMsg {})
            ),
            Ok(FirstFreeAccountResponse {
                account: Some(Account {
                    addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_1_ADDR),
                    occupied_by_job_id: None
                })
            })
        );

        // Query free accounts
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_job_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryFreeAccounts(QueryFreeAccountsMsg {
                    start_after: None,
                    limit: None
                })
            ),
            Ok(AccountsResponse {
                accounts: vec![
                    Account {
                        addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_3_ADDR),
                        occupied_by_job_id: None
                    },
                    Account {
                        addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_2_ADDR),
                        occupied_by_job_id: None
                    },
                    Account {
                        addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_1_ADDR),
                        occupied_by_job_id: None
                    }
                ],
                total_count: 3
            })
        );

        // Query occupied accounts
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_job_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryOccupiedAccounts(QueryOccupiedAccountsMsg {
                    start_after: None,
                    limit: None
                })
            ),
            Ok(AccountsResponse {
                accounts: vec![],
                total_count: 0
            })
        );

        // Occupy second account with job 1
        let _ = app.execute_contract(
            Addr::unchecked(USER_1),
            warp_job_account_tracker_contract_addr.clone(),
            &ExecuteMsg::OccupyAccount(OccupyAccountMsg {
                account_addr: DUMMY_WARP_ACCOUNT_2_ADDR.to_string(),
                job_id: DUMMY_JOB_1_ID,
            }),
            &[],
        );

        // Cannot occupy account twice
        assert_err(
            app.execute_contract(
                Addr::unchecked(USER_1),
                warp_job_account_tracker_contract_addr.clone(),
                &ExecuteMsg::OccupyAccount(OccupyAccountMsg {
                    account_addr: DUMMY_WARP_ACCOUNT_2_ADDR.to_string(),
                    job_id: DUMMY_JOB_2_ID,
                }),
                &[],
            ),
            ContractError::AccountAlreadyOccupiedError {},
        );

        // Query free accounts
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_job_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryFreeAccounts(QueryFreeAccountsMsg {
                    start_after: None,
                    limit: None
                })
            ),
            Ok(AccountsResponse {
                accounts: vec![
                    Account {
                        addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_3_ADDR),
                        occupied_by_job_id: None
                    },
                    Account {
                        addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_1_ADDR),
                        occupied_by_job_id: None
                    }
                ],
                total_count: 2
            })
        );

        // Query occupied accounts
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_job_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryOccupiedAccounts(QueryOccupiedAccountsMsg {
                    start_after: None,
                    limit: None
                })
            ),
            Ok(AccountsResponse {
                accounts: vec![Account {
                    addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_2_ADDR),
                    occupied_by_job_id: Some(DUMMY_JOB_1_ID)
                },],
                total_count: 1
            })
        );

        // Free second account
        let _ = app.execute_contract(
            Addr::unchecked(USER_1),
            warp_job_account_tracker_contract_addr.clone(),
            &ExecuteMsg::FreeAccount(FreeAccountMsg {
                account_addr: DUMMY_WARP_ACCOUNT_2_ADDR.to_string(),
            }),
            &[],
        );

        // Query free accounts
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_job_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryFreeAccounts(QueryFreeAccountsMsg {
                    start_after: None,
                    limit: None
                })
            ),
            Ok(AccountsResponse {
                accounts: vec![
                    Account {
                        addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_3_ADDR),
                        occupied_by_job_id: None
                    },
                    Account {
                        addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_2_ADDR),
                        occupied_by_job_id: None
                    },
                    Account {
                        addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_1_ADDR),
                        occupied_by_job_id: None
                    }
                ],
                total_count: 3
            })
        );

        // Query occupied accounts
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_job_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryOccupiedAccounts(QueryOccupiedAccountsMsg {
                    start_after: None,
                    limit: None
                })
            ),
            Ok(AccountsResponse {
                accounts: vec![],
                total_count: 0
            })
        );
    }
}
#[cfg(test)]
mod tests {
    use account_tracker::{
        AccountStatus, Config, ConfigResponse, ExecuteMsg, FreeJobAccountMsg, InstantiateMsg,
        JobAccount, JobAccountResponse, JobAccountsResponse, QueryConfigMsg,
        QueryFirstFreeJobAccountMsg, QueryJobAccountsMsg, QueryMsg, TakeJobAccountMsg,
    };
    use anyhow::Result as AnyResult;
    use cosmwasm_std::{Addr, Coin, Empty, Uint128, Uint64};
    use cw_multi_test::{App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor};

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

    fn contract_warp_account_tracker() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    fn init_warp_account_tracker(
        app: &mut App,
        warp_account_tracker_contract_code_id: u64,
    ) -> Addr {
        app.instantiate_contract(
            warp_account_tracker_contract_code_id,
            Addr::unchecked(DUMMY_WARP_CONTROLLER_ADDR),
            &InstantiateMsg {
                admin: USER_1.to_string(),
                warp_addr: DUMMY_WARP_CONTROLLER_ADDR.to_string(),
            },
            &[],
            "warp_account_tracker",
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
    fn warp_account_tracker_contract_multi_test_account_management() {
        let mut app = mock_app();
        let warp_account_tracker_contract_code_id = app.store_code(contract_warp_account_tracker());

        // Instantiate account
        let warp_account_tracker_contract_addr =
            init_warp_account_tracker(&mut app, warp_account_tracker_contract_code_id);
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryConfig(QueryConfigMsg {})
            ),
            Ok(ConfigResponse {
                config: Config {
                    admin: Addr::unchecked(USER_1),
                    warp_addr: Addr::unchecked(DUMMY_WARP_CONTROLLER_ADDR),
                }
            })
        );
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryFirstFreeJobAccount(QueryFirstFreeJobAccountMsg {
                    account_owner_addr: USER_1.to_string(),
                })
            ),
            Ok(JobAccountResponse { job_account: None })
        );
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryJobAccounts(QueryJobAccountsMsg {
                    account_owner_addr: USER_1.to_string(),
                    start_after: None,
                    limit: None,
                    account_status: AccountStatus::Free
                })
            ),
            Ok(JobAccountsResponse {
                job_accounts: vec![],
                total_count: 0
            })
        );
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryJobAccounts(QueryJobAccountsMsg {
                    account_owner_addr: USER_1.to_string(),
                    start_after: None,
                    limit: None,
                    account_status: AccountStatus::Taken
                })
            ),
            Ok(JobAccountsResponse {
                job_accounts: vec![],
                total_count: 0
            })
        );

        // Mark first account as free
        let _ = app.execute_contract(
            Addr::unchecked(USER_1),
            warp_account_tracker_contract_addr.clone(),
            &ExecuteMsg::FreeJobAccount(FreeJobAccountMsg {
                account_owner_addr: USER_1.to_string(),
                account_addr: DUMMY_WARP_ACCOUNT_1_ADDR.to_string(),
                last_job_id: DUMMY_JOB_1_ID,
            }),
            &[],
        );

        // free account idempotent
        let _ = app.execute_contract(
            Addr::unchecked(USER_1),
            warp_account_tracker_contract_addr.clone(),
            &ExecuteMsg::FreeJobAccount(FreeJobAccountMsg {
                account_owner_addr: USER_1.to_string(),
                account_addr: DUMMY_WARP_ACCOUNT_1_ADDR.to_string(),
                last_job_id: DUMMY_JOB_1_ID,
            }),
            &[],
        );

        // Mark second account as free
        let _ = app.execute_contract(
            Addr::unchecked(USER_1),
            warp_account_tracker_contract_addr.clone(),
            &ExecuteMsg::FreeJobAccount(FreeJobAccountMsg {
                account_owner_addr: USER_1.to_string(),
                account_addr: DUMMY_WARP_ACCOUNT_2_ADDR.to_string(),
                last_job_id: DUMMY_JOB_2_ID,
            }),
            &[],
        );

        // Mark third account as free
        let _ = app.execute_contract(
            Addr::unchecked(USER_1),
            warp_account_tracker_contract_addr.clone(),
            &ExecuteMsg::FreeJobAccount(FreeJobAccountMsg {
                account_owner_addr: USER_1.to_string(),
                account_addr: DUMMY_WARP_ACCOUNT_3_ADDR.to_string(),
                last_job_id: DUMMY_JOB_1_ID,
            }),
            &[],
        );

        // Query first free account
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryFirstFreeJobAccount(QueryFirstFreeJobAccountMsg {
                    account_owner_addr: USER_1.to_string(),
                })
            ),
            Ok(JobAccountResponse {
                job_account: Some(JobAccount {
                    account_addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_1_ADDR),
                    taken_by_job_id: DUMMY_JOB_1_ID,
                    account_status: AccountStatus::Free
                })
            })
        );

        // Query free accounts
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryJobAccounts(QueryJobAccountsMsg {
                    account_owner_addr: USER_1.to_string(),
                    start_after: None,
                    limit: None,
                    account_status: AccountStatus::Free
                })
            ),
            Ok(JobAccountsResponse {
                job_accounts: vec![
                    JobAccount {
                        account_addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_1_ADDR),
                        taken_by_job_id: DUMMY_JOB_1_ID,
                        account_status: AccountStatus::Free
                    },
                    JobAccount {
                        account_addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_2_ADDR),
                        taken_by_job_id: DUMMY_JOB_2_ID,
                        account_status: AccountStatus::Free
                    },
                    JobAccount {
                        account_addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_3_ADDR),
                        taken_by_job_id: DUMMY_JOB_1_ID,
                        account_status: AccountStatus::Free
                    },
                ],
                total_count: 3
            })
        );

        // Query taken accounts
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryJobAccounts(QueryJobAccountsMsg {
                    account_owner_addr: USER_1.to_string(),
                    start_after: None,
                    limit: None,
                    account_status: AccountStatus::Taken
                })
            ),
            Ok(JobAccountsResponse {
                job_accounts: vec![],
                total_count: 0
            })
        );

        // Take second account with job 1
        let _ = app.execute_contract(
            Addr::unchecked(USER_1),
            warp_account_tracker_contract_addr.clone(),
            &ExecuteMsg::TakeJobAccount(TakeJobAccountMsg {
                account_owner_addr: USER_1.to_string(),
                account_addr: DUMMY_WARP_ACCOUNT_2_ADDR.to_string(),
                job_id: DUMMY_JOB_1_ID,
            }),
            &[],
        );

        // Cannot take account twice
        assert_err(
            app.execute_contract(
                Addr::unchecked(USER_1),
                warp_account_tracker_contract_addr.clone(),
                &ExecuteMsg::TakeJobAccount(TakeJobAccountMsg {
                    account_owner_addr: USER_1.to_string(),
                    account_addr: DUMMY_WARP_ACCOUNT_2_ADDR.to_string(),
                    job_id: DUMMY_JOB_2_ID,
                }),
                &[],
            ),
            ContractError::AccountAlreadyTakenError {},
        );

        // Query free accounts
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryJobAccounts(QueryJobAccountsMsg {
                    account_owner_addr: USER_1.to_string(),
                    start_after: None,
                    limit: None,
                    account_status: AccountStatus::Free
                })
            ),
            Ok(JobAccountsResponse {
                job_accounts: vec![
                    JobAccount {
                        account_addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_1_ADDR),
                        taken_by_job_id: DUMMY_JOB_1_ID,
                        account_status: AccountStatus::Free
                    },
                    JobAccount {
                        account_addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_3_ADDR),
                        taken_by_job_id: DUMMY_JOB_1_ID,
                        account_status: AccountStatus::Free
                    },
                ],
                total_count: 2
            })
        );

        // Query taken accounts
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryJobAccounts(QueryJobAccountsMsg {
                    account_owner_addr: USER_1.to_string(),
                    start_after: None,
                    limit: None,
                    account_status: AccountStatus::Taken
                })
            ),
            Ok(JobAccountsResponse {
                job_accounts: vec![JobAccount {
                    account_addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_2_ADDR),
                    taken_by_job_id: DUMMY_JOB_1_ID,
                    account_status: AccountStatus::Taken
                },],
                total_count: 1
            })
        );

        // Free second account
        let _ = app.execute_contract(
            Addr::unchecked(USER_1),
            warp_account_tracker_contract_addr.clone(),
            &ExecuteMsg::FreeJobAccount(FreeJobAccountMsg {
                account_owner_addr: USER_1.to_string(),
                account_addr: DUMMY_WARP_ACCOUNT_2_ADDR.to_string(),
                last_job_id: DUMMY_JOB_1_ID,
            }),
            &[],
        );

        // Query free accounts
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_account_tracker_contract_addr.clone(),
                &QueryMsg::QueryJobAccounts(QueryJobAccountsMsg {
                    account_owner_addr: USER_1.to_string(),
                    start_after: None,
                    limit: None,
                    account_status: AccountStatus::Free
                })
            ),
            Ok(JobAccountsResponse {
                job_accounts: vec![
                    JobAccount {
                        account_addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_1_ADDR),
                        taken_by_job_id: DUMMY_JOB_1_ID,
                        account_status: AccountStatus::Free
                    },
                    JobAccount {
                        account_addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_2_ADDR),
                        taken_by_job_id: DUMMY_JOB_1_ID,
                        account_status: AccountStatus::Free
                    },
                    JobAccount {
                        account_addr: Addr::unchecked(DUMMY_WARP_ACCOUNT_3_ADDR),
                        taken_by_job_id: DUMMY_JOB_1_ID,
                        account_status: AccountStatus::Free
                    },
                ],
                total_count: 3
            })
        );

        // Query taken accounts
        assert_eq!(
            app.wrap().query_wasm_smart(
                warp_account_tracker_contract_addr,
                &QueryMsg::QueryJobAccounts(QueryJobAccountsMsg {
                    account_owner_addr: USER_1.to_string(),
                    start_after: None,
                    limit: None,
                    account_status: AccountStatus::Taken
                })
            ),
            Ok(JobAccountsResponse {
                job_accounts: vec![],
                total_count: 0
            })
        );
    }
}

use std::path::PathBuf;
use cw_orch::interface;
use cw_orch::prelude::*;
use account::{InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg};

// Provide the messages in the order Init, Exec, Query, Migrate.
#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct WarpAccount;

// Implement the Uploadable trait so it can be uploaded to the mock.
impl <Chain: CwEnv> Uploadable for WarpAccount<Chain> {
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("warp_account.wasm")
            .unwrap()
    }
    fn wrapper(&self) -> Box<dyn MockContract<Empty>> {
        Box::new(
            ContractWrapper::new_with_empty(
                crate::contract::execute,
                crate::contract::instantiate,
                crate::contract::query,
            )
                .with_migrate(crate::contract::migrate),
        )
    }
}
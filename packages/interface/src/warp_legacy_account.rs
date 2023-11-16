use cw_orch::{interface, prelude::*};

use warp_legacy_account::contract;
pub use legacy_account::{ExecuteMsg, InstantiateMsg, QueryMsg};


#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct WarpLegacyAccount;

impl<Chain: CwEnv> Uploadable for WarpLegacyAccount<Chain> {
    // Return the path to the wasm file
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("warp_legacy_account.wasm")
            .unwrap()
    }
    // Return a CosmWasm contract wrapper
    fn wrapper(&self) -> Box<dyn MockContract<Empty>> {
        Box::new(
            ContractWrapper::new_with_empty(
                contract::execute,
                contract::instantiate,
                contract::query,
            )
                .with_migrate(contract::migrate),
        )
    }
}
use cw_orch::{interface, prelude::*};

use warp_job_account_tracker::contract;
pub use job_account_tracker::{ExecuteMsg, InstantiateMsg, QueryMsg};


#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct WarpJobAccountTracker;

impl<Chain: CwEnv> Uploadable for WarpJobAccountTracker<Chain> {
    // Return the path to the wasm file
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("warp_job_account_tracker.wasm")
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
use cw_orch::{interface, prelude::*};

pub use job_account::{ExecuteMsg, InstantiateMsg, QueryMsg};
use warp_job_account::contract;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct WarpJobAccount;

impl<Chain: CwEnv> Uploadable for WarpJobAccount<Chain> {
    // Return the path to the wasm file
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("warp_job_account.wasm")
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

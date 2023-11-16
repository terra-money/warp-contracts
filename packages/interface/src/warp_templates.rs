use cw_orch::{interface, prelude::*};

pub use templates::{ExecuteMsg, InstantiateMsg, QueryMsg};
use warp_templates::contract;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct WarpTemplates;

impl<Chain: CwEnv> Uploadable for WarpTemplates<Chain> {
    // Return the path to the wasm file
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("warp_templates.wasm")
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

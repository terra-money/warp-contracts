use cw_orch::daemon::{DaemonBuilder, networks};
use cw_orch::prelude::{CwOrchInstantiate, CwOrchUpload};
use cw_orch::tokio::runtime::Runtime;
use account::{ExecuteMsgFns, GenericMsg, InstantiateMsg};
use warp_account::interface::WarpAccount;

fn main() {

    let rt = Runtime::new()?;
    let network = networks::LOCAL_TERRA;
    let chain = DaemonBuilder::default()
        .handle(rt.handle())
        .chain(network)
        .build()?;

    let contract = WarpAccount::new("warp-account", chain);

    let res = contract.upload().unwrap();
    let inst_res = contract.instantiate(
        &InstantiateMsg {
        owner: "test".to_string(),
        msgs: None,
        funds: None,
    },
        None,
        None
    ).unwrap();

    contract.generic(
        GenericMsg {
            msgs: vec![

            ],
        }
    )?;

    println!("Hello, world!");
}

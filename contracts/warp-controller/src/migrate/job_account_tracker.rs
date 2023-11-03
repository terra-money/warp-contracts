use cosmwasm_std::{to_binary, MessageInfo, Response, WasmMsg};

use crate::ContractError;
use controller::{Config, MigrateJobAccountTrackerMsg};

pub fn migrate_job_account_tracker(
    info: MessageInfo,
    msg: MigrateJobAccountTrackerMsg,
    config: Config,
) -> Result<Response, ContractError> {
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let migration_msg = WasmMsg::Migrate {
        contract_addr: config.job_account_tracker_address.to_string(),
        new_code_id: msg.warp_job_account_tracker_code_id.u64(),
        msg: to_binary(&job_account_tracker::MigrateMsg {})?,
    };
    Ok(Response::new().add_message(migration_msg))
}

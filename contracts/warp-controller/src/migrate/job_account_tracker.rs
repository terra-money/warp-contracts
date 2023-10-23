use cosmwasm_std::{to_binary, Addr, Deps, MessageInfo, Order, Response, StdResult, WasmMsg};
use cw_storage_plus::Bound;

use crate::{state::JOB_ACCOUNT_TRACKERS, ContractError};
use controller::{Config, MigrateJobAccountTrackersMsg};

pub fn migrate_job_account_trackers(
    deps: Deps,
    info: MessageInfo,
    msg: MigrateJobAccountTrackersMsg,
    config: Config,
) -> Result<Response, ContractError> {
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let job_account_tracker_keys = match msg.start_after {
        None => JOB_ACCOUNT_TRACKERS.keys(deps.storage, None, None, Order::Ascending),
        Some(start_after) => JOB_ACCOUNT_TRACKERS.keys(
            deps.storage,
            Some(Bound::exclusive(
                &deps.api.addr_validate(start_after.as_str())?,
            )),
            None,
            Order::Ascending,
        ),
    }
    .take(msg.limit as usize)
    .collect::<StdResult<Vec<Addr>>>()?;

    // let job_account_tracker_keys = job_account_tracker_keys?;
    let mut migration_msgs = vec![];

    for job_account_tracker_key in job_account_tracker_keys {
        let job_account_tracker =
            JOB_ACCOUNT_TRACKERS.load(deps.storage, &job_account_tracker_key)?;
        migration_msgs.push(WasmMsg::Migrate {
            contract_addr: job_account_tracker.to_string(),
            new_code_id: msg.warp_job_account_tracker_code_id.u64(),
            msg: to_binary(&job_account_tracker::MigrateMsg {})?,
        })
    }

    Ok(Response::new().add_messages(migration_msgs))
}

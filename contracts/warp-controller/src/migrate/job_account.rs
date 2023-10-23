use cosmwasm_std::{to_binary, Deps, Env, MessageInfo, Response, WasmMsg};

use crate::ContractError;
use controller::{Config, MigrateJobAccountsMsg};
use job_account_tracker::{
    AccountsResponse, MigrateMsg, QueryFreeAccountsMsg, QueryOccupiedAccountsMsg,
};

pub fn migrate_free_job_accounts(
    deps: Deps,
    _env: Env,
    info: MessageInfo,
    msg: MigrateJobAccountsMsg,
    config: Config,
) -> Result<Response, ContractError> {
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let free_job_accounts: AccountsResponse = deps.querier.query_wasm_smart(
        msg.job_account_tracker_addr,
        &job_account_tracker::QueryMsg::QueryFreeAccounts(QueryFreeAccountsMsg {
            start_after: msg.start_after,
            limit: Some(msg.limit as u32),
        }),
    )?;

    let mut migration_msgs = vec![];
    for job_account in free_job_accounts.accounts {
        migration_msgs.push(WasmMsg::Migrate {
            contract_addr: job_account.addr.to_string(),
            new_code_id: msg.warp_job_account_code_id.u64(),
            msg: to_binary(&MigrateMsg {})?,
        });
    }

    Ok(Response::new().add_messages(migration_msgs))
}

pub fn migrate_occupied_job_accounts(
    deps: Deps,
    _env: Env,
    info: MessageInfo,
    msg: MigrateJobAccountsMsg,
    config: Config,
) -> Result<Response, ContractError> {
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let occupied_job_accounts: AccountsResponse = deps.querier.query_wasm_smart(
        msg.job_account_tracker_addr,
        &job_account_tracker::QueryMsg::QueryOccupiedAccounts(QueryOccupiedAccountsMsg {
            start_after: msg.start_after,
            limit: Some(msg.limit as u32),
        }),
    )?;

    let mut migration_msgs = vec![];
    for job_account in occupied_job_accounts.accounts {
        migration_msgs.push(WasmMsg::Migrate {
            contract_addr: job_account.addr.to_string(),
            new_code_id: msg.warp_job_account_code_id.u64(),
            msg: to_binary(&MigrateMsg {})?,
        });
    }

    Ok(Response::new().add_messages(migration_msgs))
}

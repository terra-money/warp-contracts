use cosmwasm_std::{to_binary, Deps, Env, MessageInfo, Response, WasmMsg};

use crate::ContractError;
use controller::{Config, MigrateJobAccountsMsg};
use job_account_tracker::{
    AccountsResponse, MigrateMsg, QueryFreeAccountsMsg, QueryTakenAccountsMsg,
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
        config.job_account_tracker_address,
        &job_account_tracker::QueryMsg::QueryFreeAccounts(QueryFreeAccountsMsg {
            account_owner_addr: msg.account_owner_addr,
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

pub fn migrate_taken_job_accounts(
    deps: Deps,
    _env: Env,
    info: MessageInfo,
    msg: MigrateJobAccountsMsg,
    config: Config,
) -> Result<Response, ContractError> {
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let taken_job_accounts: AccountsResponse = deps.querier.query_wasm_smart(
        config.job_account_tracker_address,
        &job_account_tracker::QueryMsg::QueryTakenAccounts(QueryTakenAccountsMsg {
            account_owner_addr: msg.account_owner_addr,
            start_after: msg.start_after,
            limit: Some(msg.limit as u32),
        }),
    )?;

    let mut migration_msgs = vec![];
    for job_account in taken_job_accounts.accounts {
        migration_msgs.push(WasmMsg::Migrate {
            contract_addr: job_account.addr.to_string(),
            new_code_id: msg.warp_job_account_code_id.u64(),
            msg: to_binary(&MigrateMsg {})?,
        });
    }

    Ok(Response::new().add_messages(migration_msgs))
}

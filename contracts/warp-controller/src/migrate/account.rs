use cosmwasm_std::{to_binary, Deps, Env, MessageInfo, Response, WasmMsg};

use crate::ContractError;
use account_tracker::{AccountsResponse, MigrateMsg, QueryAccountsMsg};
use controller::{Config, MigrateAccountsMsg};

pub fn migrate_accounts(
    deps: Deps,
    _env: Env,
    info: MessageInfo,
    msg: MigrateAccountsMsg,
    config: Config,
) -> Result<Response, ContractError> {
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let accounts: AccountsResponse = deps.querier.query_wasm_smart(
        config.account_tracker_address,
        &account_tracker::QueryMsg::QueryAccounts(QueryAccountsMsg {
            account_owner_addr: msg.account_owner_addr,
            start_after: msg.start_after,
            limit: Some(msg.limit as u32),
        }),
    )?;

    let mut migration_msgs = vec![];
    for account in accounts.accounts {
        migration_msgs.push(WasmMsg::Migrate {
            contract_addr: account.account_addr.to_string(),
            new_code_id: msg.warp_account_code_id.u64(),
            msg: to_binary(&MigrateMsg {})?,
        });
    }

    Ok(Response::new().add_messages(migration_msgs))
}

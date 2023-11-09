use cosmwasm_std::{to_binary, DepsMut, MessageInfo, Order, Response, WasmMsg};
use cw_storage_plus::Bound;

use crate::{state::LEGACY_ACCOUNTS, ContractError};
use controller::{Config, MigrateLegacyAccountsMsg};

pub fn migrate_legacy_accounts(
    deps: DepsMut,
    info: MessageInfo,
    msg: MigrateLegacyAccountsMsg,
    config: Config,
) -> Result<Response, ContractError> {
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let start_after = match msg.start_after {
        None => None,
        Some(s) => Some(deps.api.addr_validate(s.as_str())?),
    };
    let start_after = start_after.map(Bound::exclusive);

    let account_keys: Result<Vec<_>, _> = LEGACY_ACCOUNTS()
        .keys(deps.storage, start_after, None, Order::Ascending)
        .take(msg.limit as usize)
        .collect();
    let account_keys = account_keys?;
    let mut migration_msgs = vec![];

    for account_key in account_keys {
        let account_address = LEGACY_ACCOUNTS().load(deps.storage, account_key)?.account;
        migration_msgs.push(WasmMsg::Migrate {
            contract_addr: account_address.to_string(),
            new_code_id: msg.warp_legacy_account_code_id.u64(),
            msg: to_binary(&legacy_account::MigrateMsg {})?,
        })
    }

    Ok(Response::new().add_messages(migration_msgs))
}

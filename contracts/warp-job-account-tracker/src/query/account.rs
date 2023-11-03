use cosmwasm_std::{Deps, Order, StdResult};
use cw_storage_plus::{Bound, PrefixBound};

use crate::state::{CONFIG, FREE_ACCOUNTS, TAKEN_ACCOUNTS};

use job_account_tracker::{
    Account, AccountsResponse, ConfigResponse, FirstFreeAccountResponse, QueryFirstFreeAccountMsg,
    QueryFreeAccountsMsg, QueryTakenAccountsMsg,
};

const QUERY_LIMIT: u32 = 50;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

pub fn query_first_free_account(
    deps: Deps,
    data: QueryFirstFreeAccountMsg,
) -> StdResult<FirstFreeAccountResponse> {
    let account_owner_ref = &deps.api.addr_validate(data.account_owner_addr.as_str())?;
    let maybe_free_account = FREE_ACCOUNTS
        .prefix_range(
            deps.storage,
            Some(PrefixBound::inclusive(account_owner_ref)),
            Some(PrefixBound::inclusive(account_owner_ref)),
            Order::Ascending,
        )
        .next();
    let free_account = match maybe_free_account {
        Some(Ok((account, _))) => Some(Account {
            addr: account.1,
            taken_by_job_id: None,
        }),
        _ => None,
    };
    Ok(FirstFreeAccountResponse {
        account: free_account,
    })
}

pub fn query_taken_accounts(
    deps: Deps,
    data: QueryTakenAccountsMsg,
) -> StdResult<AccountsResponse> {
    let account_owner_ref = &deps.api.addr_validate(data.account_owner_addr.as_str())?;
    let iter = match data.start_after {
        Some(start_after) => {
            let start_after_account_addr = &deps.api.addr_validate(start_after.as_str())?;
            TAKEN_ACCOUNTS.range(
                deps.storage,
                Some(Bound::exclusive((
                    account_owner_ref,
                    start_after_account_addr,
                ))),
                None,
                Order::Descending,
            )
        }
        None => TAKEN_ACCOUNTS.prefix_range(
            deps.storage,
            Some(PrefixBound::inclusive(account_owner_ref)),
            Some(PrefixBound::inclusive(account_owner_ref)),
            Order::Descending,
        ),
    };
    let accounts = iter
        .take(data.limit.unwrap_or(QUERY_LIMIT) as usize)
        .map(|item| {
            item.map(|(account, job_id)| Account {
                addr: account.1,
                taken_by_job_id: Some(job_id),
            })
        })
        .collect::<StdResult<Vec<Account>>>()?;
    Ok(AccountsResponse {
        total_count: accounts.len(),
        accounts,
    })
}

pub fn query_free_accounts(deps: Deps, data: QueryFreeAccountsMsg) -> StdResult<AccountsResponse> {
    let account_owner_ref = &deps.api.addr_validate(data.account_owner_addr.as_str())?;
    let iter = match data.start_after {
        Some(start_after) => {
            let start_after_account_addr = &deps.api.addr_validate(start_after.as_str())?;
            FREE_ACCOUNTS.range(
                deps.storage,
                Some(Bound::exclusive((
                    account_owner_ref,
                    start_after_account_addr,
                ))),
                None,
                Order::Descending,
            )
        }
        None => FREE_ACCOUNTS.prefix_range(
            deps.storage,
            Some(PrefixBound::inclusive(account_owner_ref)),
            Some(PrefixBound::inclusive(account_owner_ref)),
            Order::Descending,
        ),
    };
    let accounts = iter
        .take(data.limit.unwrap_or(QUERY_LIMIT) as usize)
        .map(|item| {
            item.map(|(account, _)| Account {
                addr: account.1,
                taken_by_job_id: None,
            })
        })
        .collect::<StdResult<Vec<Account>>>()?;
    Ok(AccountsResponse {
        total_count: accounts.len(),
        accounts,
    })
}

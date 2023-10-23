use cosmwasm_std::{Deps, Order, StdResult};
use cw_storage_plus::Bound;

use crate::state::{CONFIG, FREE_ACCOUNTS, OCCUPIED_ACCOUNTS};

use job_account_tracker::{
    Account, AccountsResponse, ConfigResponse, FirstFreeAccountResponse, QueryFreeAccountsMsg,
    QueryOccupiedAccountsMsg,
};

const QUERY_LIMIT: u32 = 50;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

pub fn query_first_free_account(deps: Deps) -> StdResult<FirstFreeAccountResponse> {
    match FREE_ACCOUNTS
        .range(deps.storage, None, None, Order::Ascending)
        .next()
    {
        Some(free_account) => Ok(FirstFreeAccountResponse {
            account: Some(Account {
                addr: free_account.unwrap().0,
                occupied_by_job_id: None,
            }),
        }),
        None => Ok(FirstFreeAccountResponse { account: None }),
    }
}

pub fn query_occupied_accounts(
    deps: Deps,
    data: QueryOccupiedAccountsMsg,
) -> StdResult<AccountsResponse> {
    let iter = match data.start_after {
        Some(start_after) => OCCUPIED_ACCOUNTS.range(
            deps.storage,
            Some(Bound::exclusive(
                &deps.api.addr_validate(start_after.as_str()).unwrap(),
            )),
            None,
            Order::Descending,
        ),
        None => OCCUPIED_ACCOUNTS.range(deps.storage, None, None, Order::Descending),
    };
    let accounts = iter
        .take(data.limit.unwrap_or(QUERY_LIMIT) as usize)
        .map(|item| {
            item.map(|(account_addr, job_id)| Account {
                addr: account_addr,
                occupied_by_job_id: Some(job_id),
            })
        })
        .collect::<StdResult<Vec<Account>>>()?;
    Ok(AccountsResponse {
        total_count: accounts.len(),
        accounts,
    })
}

pub fn query_free_accounts(deps: Deps, data: QueryFreeAccountsMsg) -> StdResult<AccountsResponse> {
    let iter = match data.start_after {
        Some(start_after) => FREE_ACCOUNTS.range(
            deps.storage,
            Some(Bound::exclusive(
                &deps.api.addr_validate(start_after.as_str()).unwrap(),
            )),
            None,
            Order::Descending,
        ),
        None => FREE_ACCOUNTS.range(deps.storage, None, None, Order::Descending),
    };
    let accounts = iter
        .take(data.limit.unwrap_or(QUERY_LIMIT) as usize)
        .map(|item| {
            item.map(|(account_addr, _)| Account {
                addr: account_addr,
                occupied_by_job_id: None,
            })
        })
        .collect::<StdResult<Vec<Account>>>()?;
    Ok(AccountsResponse {
        total_count: accounts.len(),
        accounts,
    })
}

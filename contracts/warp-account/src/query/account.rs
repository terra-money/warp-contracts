use crate::state::{CONFIG, FREE_SUB_ACCOUNTS, OCCUPIED_SUB_ACCOUNTS};
use account::{
    ConfigResponse, FirstFreeSubAccountResponse, FreeSubAccountsResponse,
    OccupiedSubAccountsResponse, QueryFreeSubAccountsMsg, QueryOccupiedSubAccountsMsg, SubAccount,
};
use cosmwasm_std::{Deps, Order, StdResult};
use cw_storage_plus::Bound;

const QUERY_LIMIT: u32 = 50;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

pub fn query_first_free_sub_account(deps: Deps) -> StdResult<FirstFreeSubAccountResponse> {
    let sub_account = FREE_SUB_ACCOUNTS
        .range(deps.storage, None, None, Order::Ascending)
        .next();
    if sub_account.is_none() {
        Ok(FirstFreeSubAccountResponse { sub_account: None })
    } else {
        let (addr, _) = sub_account.unwrap()?;
        Ok(FirstFreeSubAccountResponse {
            sub_account: Some(SubAccount {
                addr: addr.to_string(),
                occupied_by_job_id: Option::None,
            }),
        })
    }
}

pub fn query_occupied_sub_accounts(
    deps: Deps,
    data: QueryOccupiedSubAccountsMsg,
) -> StdResult<OccupiedSubAccountsResponse> {
    let iter = match data.start_after {
        Some(start_after) => OCCUPIED_SUB_ACCOUNTS.range(
            deps.storage,
            Some(Bound::exclusive(
                &deps.api.addr_validate(start_after.as_str()).unwrap(),
            )),
            None,
            Order::Descending,
        ),
        None => OCCUPIED_SUB_ACCOUNTS.range(deps.storage, None, None, Order::Descending),
    };
    let sub_accounts = iter
        .take(data.limit.unwrap_or(QUERY_LIMIT) as usize)
        .map(|item| {
            item.map(|(sub_account_addr, job_id)| SubAccount {
                addr: sub_account_addr.to_string(),
                occupied_by_job_id: Some(job_id),
            })
        })
        .collect::<StdResult<Vec<SubAccount>>>()?;
    Ok(OccupiedSubAccountsResponse {
        total_count: sub_accounts.len(),
        sub_accounts,
    })
}

pub fn query_free_sub_accounts(
    deps: Deps,
    data: QueryFreeSubAccountsMsg,
) -> StdResult<FreeSubAccountsResponse> {
    let iter = match data.start_after {
        Some(start_after) => FREE_SUB_ACCOUNTS.range(
            deps.storage,
            Some(Bound::exclusive(
                &deps.api.addr_validate(start_after.as_str()).unwrap(),
            )),
            None,
            Order::Descending,
        ),
        None => FREE_SUB_ACCOUNTS.range(deps.storage, None, None, Order::Descending),
    };
    let sub_accounts = iter
        .take(data.limit.unwrap_or(QUERY_LIMIT) as usize)
        .map(|item| {
            item.map(|(sub_account_addr, _)| SubAccount {
                addr: sub_account_addr.to_string(),
                occupied_by_job_id: Option::None,
            })
        })
        .collect::<StdResult<Vec<SubAccount>>>()?;
    Ok(FreeSubAccountsResponse {
        total_count: sub_accounts.len(),
        sub_accounts,
    })
}
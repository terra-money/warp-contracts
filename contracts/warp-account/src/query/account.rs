use crate::state::{CONFIG, FREE_SUB_ACCOUNTS, OCCUPIED_SUB_ACCOUNTS};
use account::{
    ConfigResponse, FirstFreeSubAccountsResponse, FreeSubAccountsResponse,
    IsSubAccountOwnedAndFreeResponse, IsSubAccountOwnedAndOccupiedResponse,
    OccupiedSubAccountsResponse, QueryFreeSubAccountsMsg, QueryIsSubAccountOwnedAndFreeMsg,
    QueryIsSubAccountOwnedAndOccupiedMsg, QueryOccupiedSubAccountsMsg, SubAccount,
};
use cosmwasm_std::{Deps, Order, StdResult, Uint64};
use cw_storage_plus::Bound;

const QUERY_LIMIT: u32 = 50;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

pub fn query_occupied_sub_accounts(
    deps: Deps,
    data: QueryOccupiedSubAccountsMsg,
) -> StdResult<OccupiedSubAccountsResponse> {
    let sub_accounts = OCCUPIED_SUB_ACCOUNTS
        .range(
            deps.storage,
            data.start_after.map(Bound::exclusive),
            None,
            Order::Descending,
        )
        .take(data.limit.unwrap_or(QUERY_LIMIT) as usize)
        .map(|item| {
            item.map(|(k, v)| SubAccount {
                addr: k,
                // owner: config.owner.clone(),
                // default_account_addr: env.contract.address.clone(),
                in_use_by_job_id: Some(Uint64::from(v)),
            })
        })
        .collect::<StdResult<Vec<SubAccount>>>()?;
    Ok(OccupiedSubAccountsResponse { sub_accounts })
}

pub fn query_free_sub_accounts(
    deps: Deps,
    data: QueryFreeSubAccountsMsg,
) -> StdResult<FreeSubAccountsResponse> {
    let sub_accounts = FREE_SUB_ACCOUNTS
        .range(
            deps.storage,
            data.start_after.map(Bound::exclusive),
            None,
            Order::Descending,
        )
        .take(data.limit.unwrap_or(QUERY_LIMIT) as usize)
        .map(|item| {
            item.map(|(k, _)| SubAccount {
                addr: k,
                // owner: config.owner.clone(),
                // default_account_addr: env.contract.address.clone(),
                in_use_by_job_id: Option::None,
            })
        })
        .collect::<StdResult<Vec<SubAccount>>>()?;
    Ok(FreeSubAccountsResponse { sub_accounts })
}

pub fn query_first_free_sub_account(deps: Deps) -> StdResult<FirstFreeSubAccountsResponse> {
    let sub_account = FREE_SUB_ACCOUNTS
        .range(deps.storage, None, None, Order::Ascending)
        .next();
    if sub_account.is_none() {
        return Ok(FirstFreeSubAccountsResponse {
            sub_account: None,
        });
    } else {
        let (addr, _) = sub_account.unwrap()?;
        return Ok(FirstFreeSubAccountsResponse {
            sub_account: Some(SubAccount {
                addr: addr.clone(),
                in_use_by_job_id: Option::None,
            }),
        });
    }
}

pub fn query_is_sub_account_owned_and_occupied(
    deps: Deps,
    data: QueryIsSubAccountOwnedAndOccupiedMsg,
) -> StdResult<IsSubAccountOwnedAndOccupiedResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

pub fn query_is_sub_account_owned_and_free(
    deps: Deps,
    data: QueryIsSubAccountOwnedAndFreeMsg,
) -> StdResult<IsSubAccountOwnedAndFreeResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

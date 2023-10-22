use crate::state::{FREE_SUB_ACCOUNTS, OCCUPIED_SUB_ACCOUNTS};
use account::{
    Config, ConfigResponse, FirstFreeSubAccountResponse, FreeSubAccountsResponse,
    OccupiedSubAccountsResponse, QueryFreeSubAccountsMsg, QueryOccupiedSubAccountsMsg,
    SubAccountConfig,
};
use cosmwasm_std::{Deps, Order, StdResult};
use cw_storage_plus::Bound;

const QUERY_LIMIT: u32 = 50;

pub fn query_config(config: Config) -> StdResult<ConfigResponse> {
    Ok(ConfigResponse { config })
}

pub fn query_first_free_sub_account(
    deps: Deps,
    config: Config,
) -> StdResult<FirstFreeSubAccountResponse> {
    let sub_account = FREE_SUB_ACCOUNTS
        .range(deps.storage, None, None, Order::Ascending)
        .next();
    if sub_account.is_none() {
        Ok(FirstFreeSubAccountResponse { sub_account: None })
    } else {
        let (sub_account_addr, _) = sub_account.unwrap()?;
        Ok(FirstFreeSubAccountResponse {
            sub_account: Some(Config {
                owner: config.owner,
                creator_addr: config.creator_addr,
                account_addr: sub_account_addr,
                sub_account_config: Some(SubAccountConfig {
                    main_account_addr: config.account_addr,
                    occupied_by_job_id: None,
                }),
            }),
        })
    }
}

pub fn query_occupied_sub_accounts(
    deps: Deps,
    data: QueryOccupiedSubAccountsMsg,
    config: Config,
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
            item.map(|(sub_account_addr, job_id)| Config {
                owner: config.owner.clone(),
                creator_addr: config.creator_addr.clone(),
                account_addr: sub_account_addr,
                sub_account_config: Some(SubAccountConfig {
                    main_account_addr: config.account_addr.clone(),
                    occupied_by_job_id: Some(job_id),
                }),
            })
        })
        .collect::<StdResult<Vec<Config>>>()?;
    Ok(OccupiedSubAccountsResponse {
        total_count: sub_accounts.len(),
        sub_accounts,
    })
}

pub fn query_free_sub_accounts(
    deps: Deps,
    data: QueryFreeSubAccountsMsg,
    config: Config,
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
            item.map(|(sub_account_addr, _)| Config {
                owner: config.owner.clone(),
                creator_addr: config.creator_addr.clone(),
                account_addr: sub_account_addr,
                sub_account_config: Some(SubAccountConfig {
                    main_account_addr: config.account_addr.clone(),
                    occupied_by_job_id: None,
                }),
            })
        })
        .collect::<StdResult<Vec<Config>>>()?;
    Ok(FreeSubAccountsResponse {
        total_count: sub_accounts.len(),
        sub_accounts,
    })
}

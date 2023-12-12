use cosmwasm_std::{Deps, Order, StdResult};
use cw_storage_plus::{Bound, PrefixBound};

use crate::state::{
    ACCOUNTS, CONFIG, FREE_FUNDING_ACCOUNTS, FREE_JOB_ACCOUNTS, TAKEN_FUNDING_ACCOUNTS,
    TAKEN_JOB_ACCOUNTS,
};

use account_tracker::{
    Account, AccountStatus, AccountsResponse, ConfigResponse, FundingAccount,
    FundingAccountResponse, FundingAccountsResponse, JobAccount, JobAccountResponse,
    JobAccountsResponse, QueryAccountsMsg, QueryFirstFreeFundingAccountMsg,
    QueryFirstFreeJobAccountMsg, QueryFundingAccountMsg, QueryFundingAccountsMsg,
    QueryJobAccountMsg, QueryJobAccountsMsg,
};

const QUERY_LIMIT: u32 = 50;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

pub fn query_accounts(deps: Deps, data: QueryAccountsMsg) -> StdResult<AccountsResponse> {
    let account_owner_ref = &deps.api.addr_validate(data.account_owner_addr.as_str())?;
    let start_after = data
        .start_after
        .map(|addr| deps.api.addr_validate(&addr))
        .transpose()?;

    let iter = match start_after {
        Some(start_after_addr) => ACCOUNTS.range(
            deps.storage,
            Some(Bound::exclusive((account_owner_ref, &start_after_addr))),
            None,
            Order::Ascending,
        ),
        None => ACCOUNTS.prefix_range(
            deps.storage,
            Some(PrefixBound::inclusive(account_owner_ref)),
            Some(PrefixBound::inclusive(account_owner_ref)),
            Order::Ascending,
        ),
    };

    let accounts = iter
        .take(data.limit.unwrap_or(QUERY_LIMIT) as usize)
        .map(|item| item.map(|(_, account)| account))
        .collect::<StdResult<Vec<Account>>>()?;

    Ok(AccountsResponse { accounts })
}

pub fn query_funding_account(
    deps: Deps,
    data: QueryFundingAccountMsg,
) -> StdResult<FundingAccountResponse> {
    let account_owner_addr_ref = &deps.api.addr_validate(data.account_owner_addr.as_str())?;
    let account_addr_ref = &deps.api.addr_validate(data.account_addr.as_str())?;

    let funding_account = match TAKEN_FUNDING_ACCOUNTS
        .may_load(deps.storage, (account_owner_addr_ref, account_addr_ref))
    {
        Ok(Some(job_ids)) => Some(FundingAccount {
            account_addr: account_addr_ref.clone(),
            taken_by_job_ids: job_ids,
            account_status: AccountStatus::Taken,
        }),
        Ok(None) => {
            match FREE_FUNDING_ACCOUNTS
                .may_load(deps.storage, (account_owner_addr_ref, account_addr_ref))
            {
                Ok(Some(job_ids)) => Some(FundingAccount {
                    account_addr: account_addr_ref.clone(),
                    taken_by_job_ids: job_ids,
                    account_status: AccountStatus::Free,
                }),
                Ok(None) => None,
                Err(err) => return Err(err),
            }
        }
        Err(err) => return Err(err),
    };

    Ok(FundingAccountResponse { funding_account })
}

pub fn query_first_free_funding_account(
    deps: Deps,
    data: QueryFirstFreeFundingAccountMsg,
) -> StdResult<FundingAccountResponse> {
    let resp = query_funding_accounts(
        deps,
        QueryFundingAccountsMsg {
            account_owner_addr: data.account_owner_addr,
            account_status: AccountStatus::Free,
            start_after: None,
            limit: Some(1),
        },
    )?;

    Ok(FundingAccountResponse {
        funding_account: resp.funding_accounts.first().cloned(),
    })
}

pub fn query_funding_accounts(
    deps: Deps,
    data: QueryFundingAccountsMsg,
) -> StdResult<FundingAccountsResponse> {
    let account_owner_ref = &deps.api.addr_validate(data.account_owner_addr.as_str())?;
    let status = data.account_status;

    let iter = match status {
        AccountStatus::Free => match data.start_after {
            Some(start_after) => {
                let start_after_account_addr = &deps.api.addr_validate(start_after.as_str())?;
                FREE_FUNDING_ACCOUNTS.range(
                    deps.storage,
                    Some(Bound::exclusive((
                        account_owner_ref,
                        start_after_account_addr,
                    ))),
                    None,
                    Order::Ascending,
                )
            }
            None => FREE_FUNDING_ACCOUNTS.prefix_range(
                deps.storage,
                Some(PrefixBound::inclusive(account_owner_ref)),
                Some(PrefixBound::inclusive(account_owner_ref)),
                Order::Ascending,
            ),
        },
        AccountStatus::Taken => match data.start_after {
            Some(start_after) => {
                let start_after_account_addr = &deps.api.addr_validate(start_after.as_str())?;
                TAKEN_FUNDING_ACCOUNTS.range(
                    deps.storage,
                    Some(Bound::exclusive((
                        account_owner_ref,
                        start_after_account_addr,
                    ))),
                    None,
                    Order::Ascending,
                )
            }
            None => TAKEN_FUNDING_ACCOUNTS.prefix_range(
                deps.storage,
                Some(PrefixBound::inclusive(account_owner_ref)),
                Some(PrefixBound::inclusive(account_owner_ref)),
                Order::Ascending,
            ),
        },
    };

    let funding_accounts = iter
        .take(data.limit.unwrap_or(QUERY_LIMIT) as usize)
        .map(|item| {
            item.map(|(account, job_ids)| FundingAccount {
                account_addr: account.1,
                taken_by_job_ids: job_ids,
                account_status: status.clone(),
            })
        })
        .collect::<StdResult<Vec<FundingAccount>>>()?;

    Ok(FundingAccountsResponse {
        funding_accounts: funding_accounts.clone(),
        total_count: funding_accounts.len() as u32,
    })
}

pub fn query_job_accounts(deps: Deps, data: QueryJobAccountsMsg) -> StdResult<JobAccountsResponse> {
    let account_owner_ref = &deps.api.addr_validate(data.account_owner_addr.as_str())?;
    let status = data.account_status;

    let iter = match status {
        AccountStatus::Free => match data.start_after {
            Some(start_after) => {
                let start_after_account_addr = &deps.api.addr_validate(start_after.as_str())?;

                FREE_JOB_ACCOUNTS.range(
                    deps.storage,
                    Some(Bound::exclusive((
                        account_owner_ref,
                        start_after_account_addr,
                    ))),
                    None,
                    Order::Ascending,
                )
            }
            None => FREE_JOB_ACCOUNTS.prefix_range(
                deps.storage,
                Some(PrefixBound::inclusive(account_owner_ref)),
                Some(PrefixBound::inclusive(account_owner_ref)),
                Order::Ascending,
            ),
        },
        AccountStatus::Taken => match data.start_after {
            Some(start_after) => {
                let start_after_account_addr = &deps.api.addr_validate(start_after.as_str())?;

                TAKEN_JOB_ACCOUNTS.range(
                    deps.storage,
                    Some(Bound::exclusive((
                        account_owner_ref,
                        start_after_account_addr,
                    ))),
                    None,
                    Order::Ascending,
                )
            }
            None => TAKEN_JOB_ACCOUNTS.prefix_range(
                deps.storage,
                Some(PrefixBound::inclusive(account_owner_ref)),
                Some(PrefixBound::inclusive(account_owner_ref)),
                Order::Ascending,
            ),
        },
    };

    let job_accounts = iter
        .take(data.limit.unwrap_or(QUERY_LIMIT) as usize)
        .map(|item| {
            item.map(|(account, job_id)| JobAccount {
                account_addr: account.1,
                taken_by_job_id: job_id,
                account_status: status.clone(),
            })
        })
        .collect::<StdResult<Vec<JobAccount>>>()?;

    Ok(JobAccountsResponse {
        job_accounts: job_accounts.clone(),
        total_count: job_accounts.len() as u32,
    })
}

pub fn query_job_account(deps: Deps, data: QueryJobAccountMsg) -> StdResult<JobAccountResponse> {
    let account_owner_addr_ref = &deps.api.addr_validate(data.account_owner_addr.as_str())?;
    let account_addr_ref = &deps.api.addr_validate(data.account_addr.as_str())?;

    let job_account = match TAKEN_JOB_ACCOUNTS
        .may_load(deps.storage, (account_owner_addr_ref, account_addr_ref))
    {
        Ok(Some(job_id)) => Some(JobAccount {
            account_addr: account_addr_ref.clone(),
            taken_by_job_id: job_id,
            account_status: AccountStatus::Taken,
        }),
        Ok(None) => {
            match FREE_JOB_ACCOUNTS
                .may_load(deps.storage, (account_owner_addr_ref, account_addr_ref))
            {
                Ok(Some(job_id)) => Some(JobAccount {
                    account_addr: account_addr_ref.clone(),
                    taken_by_job_id: job_id,
                    account_status: AccountStatus::Free,
                }),
                Ok(None) => None,
                Err(err) => return Err(err),
            }
        }
        Err(err) => return Err(err),
    };

    Ok(JobAccountResponse { job_account })
}

pub fn query_first_free_job_account(
    deps: Deps,
    data: QueryFirstFreeJobAccountMsg,
) -> StdResult<JobAccountResponse> {
    let resp = query_job_accounts(
        deps,
        QueryJobAccountsMsg {
            account_owner_addr: data.account_owner_addr,
            account_status: AccountStatus::Free,
            start_after: None,
            limit: Some(1),
        },
    )?;

    Ok(JobAccountResponse {
        job_account: resp.job_accounts.first().cloned(),
    })
}

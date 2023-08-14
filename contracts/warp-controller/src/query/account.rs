use crate::state::{ACCOUNTS, FINISHED_JOBS, PENDING_JOBS, QUERY_PAGE_SIZE};
use controller::account::{
    Account, AccountResponse, AccountUsedByJobResponse, AccountsResponse, QueryAccountMsg,
    QueryAccountUsedByJobMsg, QueryAccountsMsg,
};
use cosmwasm_std::{Deps, Env, Order, StdError, StdResult};
use cw_storage_plus::Bound;

pub fn query_account(deps: Deps, _env: Env, data: QueryAccountMsg) -> StdResult<AccountResponse> {
    Ok(AccountResponse {
        account: ACCOUNTS().load(deps.storage, deps.api.addr_validate(data.owner.as_str())?)?,
    })
}

pub fn query_accounts(
    deps: Deps,
    _env: Env,
    data: QueryAccountsMsg,
) -> StdResult<AccountsResponse> {
    let start_after = match data.start_after {
        None => None,
        Some(s) => Some(deps.api.addr_validate(s.as_str())?),
    };
    let start_after = start_after.map(Bound::exclusive);
    let infos = ACCOUNTS()
        .range(deps.storage, start_after, None, Order::Ascending)
        .take(data.limit.unwrap_or(QUERY_PAGE_SIZE) as usize)
        .collect::<StdResult<Vec<_>>>()?;
    let mut accounts = vec![];
    for tuple in infos {
        accounts.push(tuple.1)
    }
    Ok(AccountsResponse { accounts })
}

pub fn query_account_used_by_job(
    deps: Deps,
    _env: Env,
    data: QueryAccountUsedByJobMsg,
) -> StdResult<AccountUsedByJobResponse> {
    let job = if FINISHED_JOBS().has(deps.storage, data.job_id.u64()) {
        FINISHED_JOBS().load(deps.storage, data.job_id.u64())?
    } else {
        PENDING_JOBS().load(deps.storage, data.job_id.u64())?
    };
    if job.account.is_some() {
        Ok(AccountUsedByJobResponse {
            account: Account {
                owner: job.owner,
                account: job.account.unwrap(),
            },
        })
    } else {
        if ACCOUNTS().has(deps.storage, job.owner.clone()) {
            let account = ACCOUNTS().load(deps.storage, job.owner.clone())?;
            Ok(AccountUsedByJobResponse { account })
        } else {
            Err(StdError::GenericErr {
                msg: "Job does not have an associated account, this job is corrupted!".to_string(),
            })
        }
    }
}

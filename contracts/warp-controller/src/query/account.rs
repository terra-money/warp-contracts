use crate::state::{ACCOUNTS, QUERY_PAGE_SIZE};
use cosmwasm_std::{Deps, Env, Order, StdResult};
use cw_storage_plus::Bound;
use controller::account::{
    AccountResponse, AccountsResponse, QueryAccountMsg, QueryAccountsMsg,
};

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

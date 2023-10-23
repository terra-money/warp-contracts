use cosmwasm_std::{Deps, Env, Order, StdResult};
use cw_storage_plus::Bound;

use crate::state::{ACCOUNTS, QUERY_PAGE_SIZE};

use controller::account::{
    MainAccountResponse, MainAccountsResponse, QueryMainAccountMsg, QueryMainAccountsMsg,
};

pub fn query_main_account(
    deps: Deps,
    _env: Env,
    data: QueryMainAccountMsg,
) -> StdResult<MainAccountResponse> {
    Ok(MainAccountResponse {
        main_account: ACCOUNTS()
            .load(deps.storage, deps.api.addr_validate(data.owner.as_str())?)?,
    })
}

pub fn query_main_accounts(
    deps: Deps,
    _env: Env,
    data: QueryMainAccountsMsg,
) -> StdResult<MainAccountsResponse> {
    let start_after = match data.start_after {
        None => None,
        Some(s) => Some(deps.api.addr_validate(s.as_str())?),
    };
    let start_after = start_after.map(Bound::exclusive);
    let infos = ACCOUNTS()
        .range(deps.storage, start_after, None, Order::Ascending)
        .take(data.limit.unwrap_or(QUERY_PAGE_SIZE) as usize)
        .collect::<StdResult<Vec<_>>>()?;
    let mut main_accounts = vec![];
    for tuple in infos {
        main_accounts.push(tuple.1)
    }
    Ok(MainAccountsResponse { main_accounts })
}

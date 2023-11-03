use crate::state::{FREE_ACCOUNTS, TAKEN_ACCOUNTS};
use crate::ContractError;
use cosmwasm_std::{DepsMut, Response};
use job_account_tracker::{FreeAccountMsg, TakeAccountMsg};

pub fn taken_account(deps: DepsMut, data: TakeAccountMsg) -> Result<Response, ContractError> {
    let account_addr_ref = &deps.api.addr_validate(data.account_addr.as_str())?;
    FREE_ACCOUNTS.remove(deps.storage, account_addr_ref);
    TAKEN_ACCOUNTS.update(deps.storage, account_addr_ref, |s| match s {
        None => Ok(data.job_id),
        Some(_) => Err(ContractError::AccountAlreadyOccupiedError {}),
    })?;
    Ok(Response::new()
        .add_attribute("action", "taken_account")
        .add_attribute("account_addr", data.account_addr)
        .add_attribute("job_id", data.job_id))
}

pub fn free_account(deps: DepsMut, data: FreeAccountMsg) -> Result<Response, ContractError> {
    let account_addr_ref = &deps.api.addr_validate(data.account_addr.as_str())?;
    TAKEN_ACCOUNTS.remove(deps.storage, account_addr_ref);
    FREE_ACCOUNTS.update(deps.storage, account_addr_ref, |s| match s {
        // value is a dummy data because there is no built in support for set in cosmwasm
        None => Ok(true),
        Some(_) => Err(ContractError::AccountAlreadyFreeError {}),
    })?;
    Ok(Response::new()
        .add_attribute("action", "free_account")
        .add_attribute("account_addr", data.account_addr))
}

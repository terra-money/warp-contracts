use crate::state::{FREE_SUB_ACCOUNTS, OCCUPIED_SUB_ACCOUNTS};
use crate::ContractError;
use account::{FreeSubAccountMsg, OccupySubAccountMsg};
use cosmwasm_std::{DepsMut, Env, Response};

pub fn occupy_sub_account(
    deps: DepsMut,
    env: Env,
    data: OccupySubAccountMsg,
) -> Result<Response, ContractError> {
    // We do not add main account to occupied sub accounts
    if data.sub_account_addr == env.contract.address {
        return Ok(Response::new());
    }
    FREE_SUB_ACCOUNTS.remove(deps.storage, data.sub_account_addr.clone());
    OCCUPIED_SUB_ACCOUNTS.update(deps.storage, data.sub_account_addr.clone(), |s| match s {
        None => Ok(data.job_id.u64()),
        Some(_) => Err(ContractError::SubAccountAlreadyOccupiedError {}),
    })?;
    Ok(Response::new()
        .add_attribute("action", "occupy_sub_account")
        .add_attribute("sub_account_addr", data.sub_account_addr)
        .add_attribute("job_id", data.job_id))
}

pub fn free_sub_account(
    deps: DepsMut,
    env: Env,
    data: FreeSubAccountMsg,
) -> Result<Response, ContractError> {
    // We do not add main account to free sub accounts
    if data.sub_account_addr == env.contract.address {
        return Ok(Response::new());
    }
    OCCUPIED_SUB_ACCOUNTS.remove(deps.storage, data.sub_account_addr.clone());
    FREE_SUB_ACCOUNTS.update(deps.storage, data.sub_account_addr.clone(), |s| match s {
        // value is a dummy data because there is no built in support for set in cosmwasm
        None => Ok(0),
        Some(_) => Err(ContractError::SubAccountAlreadyFreeError {}),
    })?;
    Ok(Response::new()
        .add_attribute("action", "free_sub_account")
        .add_attribute("sub_account_addr", data.sub_account_addr))
}

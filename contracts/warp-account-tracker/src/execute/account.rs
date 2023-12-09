use crate::state::{
    FREE_ACCOUNTS, FUNDING_ACCOUNTS_BY_USER, TAKEN_ACCOUNTS, TAKEN_FUNDING_ACCOUNT_BY_JOB,
};
use crate::ContractError;
use cosmwasm_std::{DepsMut, Response};
use account_tracker::{
    AddFundingAccountMsg, FreeAccountMsg, FreeFundingAccountMsg, FundingAccount, TakeAccountMsg,
    TakeFundingAccountMsg,
};

pub fn taken_account(deps: DepsMut, data: TakeAccountMsg) -> Result<Response, ContractError> {
    let account_owner_ref = &deps.api.addr_validate(data.account_owner_addr.as_str())?;
    let account_addr_ref = &deps.api.addr_validate(data.account_addr.as_str())?;
    FREE_ACCOUNTS.remove(deps.storage, (account_owner_ref, account_addr_ref));
    TAKEN_ACCOUNTS.update(
        deps.storage,
        (account_owner_ref, account_addr_ref),
        |s| match s {
            None => Ok(data.job_id),
            Some(_) => Err(ContractError::AccountAlreadyTakenError {}),
        },
    )?;
    Ok(Response::new()
        .add_attribute("action", "taken_account")
        .add_attribute("account_addr", data.account_addr)
        .add_attribute("job_id", data.job_id))
}

pub fn free_account(deps: DepsMut, data: FreeAccountMsg) -> Result<Response, ContractError> {
    let account_owner_ref = &deps.api.addr_validate(data.account_owner_addr.as_str())?;
    let account_addr_ref = &deps.api.addr_validate(data.account_addr.as_str())?;
    TAKEN_ACCOUNTS.remove(deps.storage, (account_owner_ref, account_addr_ref));
    FREE_ACCOUNTS.update(
        deps.storage,
        (account_owner_ref, account_addr_ref),
        |s| match s {
            // value is a dummy data because there is no built in support for set in cosmwasm
            None => Ok(data.last_job_id),
            Some(_) => Err(ContractError::AccountAlreadyFreeError {}),
        },
    )?;
    Ok(Response::new()
        .add_attribute("action", "free_account")
        .add_attribute("account_addr", data.account_addr))
}

pub fn take_funding_account(
    deps: DepsMut,
    data: TakeFundingAccountMsg,
) -> Result<Response, ContractError> {
    let account_owner_addr_ref = deps.api.addr_validate(&data.account_owner_addr)?;
    let account_addr_ref = &deps.api.addr_validate(data.account_addr.as_str())?;

    // prevent taking job accounts as funding accounts
    if TAKEN_ACCOUNTS.has(deps.storage, (&account_owner_addr_ref, account_addr_ref))
        || FREE_ACCOUNTS.has(deps.storage, (&account_owner_addr_ref, account_addr_ref))
    {
        return Err(ContractError::AccountAlreadyTakenError {});
    }

    TAKEN_FUNDING_ACCOUNT_BY_JOB.update(deps.storage, data.job_id.u64(), |s| match s {
        // value is a dummy data because there is no built in support for set in cosmwasm
        None => Ok(account_addr_ref.clone()),
        Some(_) => Err(ContractError::AccountAlreadyTakenError {}),
    })?;

    FUNDING_ACCOUNTS_BY_USER.update(
        deps.storage,
        &account_owner_addr_ref,
        |accounts_opt| -> Result<Vec<FundingAccount>, ContractError> {
            match accounts_opt {
                None => {
                    // No funding accounts exist for this user, create a new vec
                    Ok(vec![FundingAccount {
                        account_addr: account_addr_ref.clone(),
                        taken_by_job_ids: vec![data.job_id],
                    }])
                }
                Some(mut accounts) => {
                    // Check if a funding account with the specified address already exists
                    if let Some(funding_account) = accounts
                        .iter_mut()
                        .find(|acc| acc.account_addr == account_addr_ref.clone())
                    {
                        // Funding account exists, update its job_ids
                        funding_account.taken_by_job_ids.push(data.job_id);
                    } else {
                        // Funding account does not exist, add a new one
                        accounts.push(FundingAccount {
                            account_addr: account_addr_ref.clone(),
                            taken_by_job_ids: vec![data.job_id],
                        });
                    }
                    Ok(accounts)
                }
            }
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "take_funding_account")
        .add_attribute("account_addr", data.account_addr)
        .add_attribute("job_id", data.job_id.to_string()))
}

pub fn free_funding_account(
    deps: DepsMut,
    data: FreeFundingAccountMsg,
) -> Result<Response, ContractError> {
    let account_owner_addr_ref = deps.api.addr_validate(&data.account_owner_addr)?;
    let account_addr_ref = deps.api.addr_validate(&data.account_addr)?;

    TAKEN_FUNDING_ACCOUNT_BY_JOB.remove(deps.storage, data.job_id.u64());

    FUNDING_ACCOUNTS_BY_USER.update(
        deps.storage,
        &account_owner_addr_ref,
        |accounts_opt| -> Result<Vec<FundingAccount>, ContractError> {
            match accounts_opt {
                Some(mut accounts) => {
                    // Find the funding account with the specified address
                    if let Some(funding_account) = accounts
                        .iter_mut()
                        .find(|acc| acc.account_addr == account_addr_ref)
                    {
                        // Remove the specified job ID
                        funding_account
                            .taken_by_job_ids
                            .retain(|&id| id != data.job_id);

                        Ok(accounts)
                    } else {
                        Err(ContractError::AccountNotFound {})
                    }
                }
                None => Err(ContractError::AccountNotFound {}),
            }
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "free_funding_account")
        .add_attribute("account_addr", data.account_addr)
        .add_attribute("job_id", data.job_id.to_string()))
}

pub fn add_funding_account(
    deps: DepsMut,
    data: AddFundingAccountMsg,
) -> Result<Response, ContractError> {
    let account_owner_addr_ref = deps.api.addr_validate(&data.account_owner_addr)?;
    let account_addr_ref = deps.api.addr_validate(&data.account_addr)?;

    // prevent adding job accounts as funding accounts
    if TAKEN_ACCOUNTS.has(deps.storage, (&account_owner_addr_ref, &account_addr_ref))
        || FREE_ACCOUNTS.has(deps.storage, (&account_owner_addr_ref, &account_addr_ref))
    {
        return Err(ContractError::AccountAlreadyTakenError {});
    }

    FUNDING_ACCOUNTS_BY_USER.update(
        deps.storage,
        &account_owner_addr_ref,
        |accounts_opt| -> Result<Vec<FundingAccount>, ContractError> {
            match accounts_opt {
                Some(mut accounts) => {
                    if accounts
                        .iter_mut()
                        .any(|acc| acc.account_addr == account_addr_ref.clone())
                    {
                        // account already exists, do nothing
                        Ok(accounts)
                    } else {
                        accounts.push(FundingAccount {
                            account_addr: account_addr_ref.clone(),
                            taken_by_job_ids: vec![],
                        });

                        Ok(accounts)
                    }
                }
                None => Ok(vec![FundingAccount {
                    account_addr: account_addr_ref.clone(),
                    taken_by_job_ids: vec![],
                }]),
            }
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "add_funding_account")
        .add_attribute("account_addr", data.account_addr))
}
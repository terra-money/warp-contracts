use crate::state::{
    ACCOUNTS, FREE_FUNDING_ACCOUNTS, FREE_JOB_ACCOUNTS, TAKEN_FUNDING_ACCOUNTS, TAKEN_JOB_ACCOUNTS,
};
use crate::ContractError;
use account_tracker::{
    Account, AccountType, FreeFundingAccountMsg, FreeJobAccountMsg, TakeFundingAccountMsg,
    TakeJobAccountMsg,
};
use cosmwasm_std::{DepsMut, Response, Uint64};

pub fn take_job_account(deps: DepsMut, data: TakeJobAccountMsg) -> Result<Response, ContractError> {
    let account_owner_ref = &deps.api.addr_validate(data.account_owner_addr.as_str())?;
    let account_addr_ref = &deps.api.addr_validate(data.account_addr.as_str())?;

    // Attempt to load the account; if it doesn't exist, create a new one
    let account = ACCOUNTS.update(
        deps.storage,
        (account_owner_ref, account_addr_ref),
        |s| -> Result<Account, ContractError> {
            match s {
                Some(account) => Ok(account),
                None => Ok(Account {
                    account_type: AccountType::Job,
                    owner_addr: account_owner_ref.clone(),
                    account_addr: account_addr_ref.clone(),
                }),
            }
        },
    )?;

    if account.account_type != AccountType::Job {
        return Err(ContractError::InvalidAccountType {});
    }

    FREE_JOB_ACCOUNTS.remove(deps.storage, (account_owner_ref, account_addr_ref));
    TAKEN_JOB_ACCOUNTS.update(
        deps.storage,
        (account_owner_ref, account_addr_ref),
        |s| match s {
            None => Ok(data.job_id),
            Some(_) => Err(ContractError::AccountAlreadyTakenError {}),
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "take_job_account")
        .add_attribute("account_addr", data.account_addr)
        .add_attribute("job_id", data.job_id.to_string()))
}

pub fn free_job_account(deps: DepsMut, data: FreeJobAccountMsg) -> Result<Response, ContractError> {
    let account_owner_ref = &deps.api.addr_validate(data.account_owner_addr.as_str())?;
    let account_addr_ref = &deps.api.addr_validate(data.account_addr.as_str())?;

    // Attempt to load the account; if it doesn't exist, create a new one
    let account = ACCOUNTS.update(
        deps.storage,
        (account_owner_ref, account_addr_ref),
        |s| -> Result<Account, ContractError> {
            match s {
                Some(account) => Ok(account),
                None => Ok(Account {
                    account_type: AccountType::Job,
                    owner_addr: account_owner_ref.clone(),
                    account_addr: account_addr_ref.clone(),
                }),
            }
        },
    )?;

    if account.account_type != AccountType::Job {
        return Err(ContractError::InvalidAccountType {});
    }

    TAKEN_JOB_ACCOUNTS.remove(deps.storage, (account_owner_ref, account_addr_ref));
    FREE_JOB_ACCOUNTS.update(
        deps.storage,
        (account_owner_ref, account_addr_ref),
        |s| match s {
            None => Ok(data.last_job_id),
            Some(_) => Err(ContractError::AccountAlreadyFreeError {}),
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "free_job_account")
        .add_attribute("account_addr", data.account_addr))
}

pub fn take_funding_account(
    deps: DepsMut,
    data: TakeFundingAccountMsg,
) -> Result<Response, ContractError> {
    let account_owner_addr_ref = &deps.api.addr_validate(&data.account_owner_addr)?;
    let account_addr_ref = &deps.api.addr_validate(data.account_addr.as_str())?;

    // Attempt to load the account; if it doesn't exist, create a new one
    let account = ACCOUNTS.update(
        deps.storage,
        (account_owner_addr_ref, account_addr_ref),
        |s| -> Result<Account, ContractError> {
            match s {
                Some(account) => Ok(account),
                None => Ok(Account {
                    account_type: AccountType::Job,
                    owner_addr: account_owner_addr_ref.clone(),
                    account_addr: account_addr_ref.clone(),
                }),
            }
        },
    )?;

    if account.account_type != AccountType::Funding {
        return Err(ContractError::InvalidAccountType {});
    }

    FREE_FUNDING_ACCOUNTS.remove(deps.storage, (account_owner_addr_ref, account_addr_ref));
    TAKEN_FUNDING_ACCOUNTS.update(
        deps.storage,
        (account_owner_addr_ref, account_addr_ref),
        |ids| -> Result<Vec<Uint64>, ContractError> {
            match ids {
                Some(mut id_list) => {
                    id_list.push(data.job_id);
                    Ok(id_list)
                }
                None => Ok(vec![data.job_id]),
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
    let account_owner_addr_ref = &deps.api.addr_validate(&data.account_owner_addr)?;
    let account_addr_ref = &deps.api.addr_validate(&data.account_addr)?;

    // Attempt to load the account; if it doesn't exist, create a new one
    let account = ACCOUNTS.update(
        deps.storage,
        (account_owner_addr_ref, account_addr_ref),
        |s| -> Result<Account, ContractError> {
            match s {
                Some(account) => Ok(account),
                None => Ok(Account {
                    account_type: AccountType::Job,
                    owner_addr: account_owner_addr_ref.clone(),
                    account_addr: account_addr_ref.clone(),
                }),
            }
        },
    )?;

    if account.account_type != AccountType::Funding {
        return Err(ContractError::InvalidAccountType {});
    }

    // Retrieve current job IDs for the funding account
    let mut job_ids =
        TAKEN_FUNDING_ACCOUNTS.load(deps.storage, (account_owner_addr_ref, account_addr_ref))?;

    // Remove the specified job ID
    job_ids.retain(|&id| id != data.job_id);

    // Update or remove the entry in TAKEN_FUNDING_ACCOUNTS based on the updated list
    if job_ids.is_empty() {
        TAKEN_FUNDING_ACCOUNTS.remove(deps.storage, (account_owner_addr_ref, account_addr_ref));
        FREE_FUNDING_ACCOUNTS.update(
            deps.storage,
            (account_owner_addr_ref, account_addr_ref),
            |s| match s {
                None => Ok(vec![data.job_id]),
                Some(_) => Err(ContractError::AccountAlreadyFreeError {}),
            },
        )?;
    } else {
        // Update the entry in TAKEN_FUNDING_ACCOUNTS
        TAKEN_FUNDING_ACCOUNTS.save(
            deps.storage,
            (account_owner_addr_ref, account_addr_ref),
            &job_ids,
        )?;
    }

    Ok(Response::new()
        .add_attribute("action", "free_funding_account")
        .add_attribute("account_addr", data.account_addr)
        .add_attribute("job_id", data.job_id.to_string()))
}

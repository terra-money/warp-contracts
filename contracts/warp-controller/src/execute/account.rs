use crate::contract::{
    REPLY_ID_CREATE_ACCOUNT, REPLY_ID_CREATE_ACCOUNT_AND_JOB, REPLY_ID_CREATE_JOB_ACCOUNT_AND_JOB,
};
use crate::state::{ACCOUNTS, CONFIG, PENDING_JOBS, STATE};
use crate::ContractError;
use account::GenericMsg;
use controller::account::{
    CreateAccountAndJobMsg, CreateAccountMsg, Fund, FundTransferMsgs, TransferFromMsg,
    TransferNftMsg,
};
use controller::State;

use controller::job::{Job, JobStatus};
use cosmwasm_std::{
    to_binary, Attribute, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, ReplyOn, Response,
    SubMsg, Uint128, Uint64, WasmMsg,
};

pub fn create_account(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: CreateAccountMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let item = ACCOUNTS()
        .idx
        .account
        .item(deps.storage, info.sender.clone());

    if item?.is_some() {
        return Err(ContractError::AccountCannotCreateAccount {});
    }

    if ACCOUNTS().has(deps.storage, info.sender.clone()) {
        let account = ACCOUNTS().load(deps.storage, info.sender.clone())?;

        let cw_funds_vec = match data.funds {
            None => {
                vec![]
            }
            Some(funds) => funds,
        };

        let mut msgs_vec: Vec<CosmosMsg> = vec![];

        if !info.funds.is_empty() {
            msgs_vec.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: account.account.to_string(),
                amount: info.funds.clone(),
            }))
        }

        for cw_fund in &cw_funds_vec {
            msgs_vec.push(CosmosMsg::Wasm(match cw_fund {
                Fund::Cw20(cw20_fund) => WasmMsg::Execute {
                    contract_addr: deps
                        .api
                        .addr_validate(&cw20_fund.contract_addr)?
                        .to_string(),
                    msg: to_binary(&FundTransferMsgs::TransferFrom(TransferFromMsg {
                        owner: info.sender.clone().to_string(),
                        recipient: account.account.clone().to_string(),
                        amount: cw20_fund.amount,
                    }))?,
                    funds: vec![],
                },
                Fund::Cw721(cw721_fund) => WasmMsg::Execute {
                    contract_addr: deps
                        .api
                        .addr_validate(&cw721_fund.contract_addr)?
                        .to_string(),
                    msg: to_binary(&FundTransferMsgs::TransferNft(TransferNftMsg {
                        recipient: account.account.clone().to_string(),
                        token_id: cw721_fund.token_id.clone(),
                    }))?,
                    funds: vec![],
                },
            }))
        }

        return Ok(Response::new()
            .add_attribute("action", "create_account")
            .add_attribute("owner", account.owner)
            .add_attribute("account_address", account.account)
            .add_messages(msgs_vec));
    }

    let submsg = SubMsg {
        id: REPLY_ID_CREATE_ACCOUNT,
        msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: config.warp_account_code_id.u64(),
            msg: to_binary(&account::InstantiateMsg {
                owner: info.sender.to_string(),
                funds: data.funds,
                job_id: None,
                is_job_account: None,
            })?,
            funds: info.funds,
            label: info.sender.to_string(),
        }),
        gas_limit: None,
        reply_on: ReplyOn::Always,
    };

    Ok(Response::new()
        .add_attribute("action", "create_account")
        .add_submessage(submsg))
}

pub fn create_account_and_job(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: CreateAccountAndJobMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if data.name.len() > 280 {
        return Err(ContractError::NameTooLong {});
    }

    if data.name.is_empty() {
        return Err(ContractError::NameTooShort {});
    }

    if data.reward < config.minimum_reward || data.reward.is_zero() {
        return Err(ContractError::RewardTooSmall {});
    }

    let _validate_conditions_and_variables: Option<String> = deps.querier.query_wasm_smart(
        config.resolver_address,
        &resolver::QueryMsg::QueryValidateJobCreation(resolver::QueryValidateJobCreationMsg {
            condition: data.condition.clone(),
            terminate_condition: data.terminate_condition.clone(),
            vars: data.vars.clone(),
            msgs: data.msgs.clone(),
        }),
    )?; //todo: TEST THIS

    let config = CONFIG.load(deps.storage)?;

    let item = ACCOUNTS()
        .idx
        .account
        .item(deps.storage, info.sender.clone());

    if item?.is_some() {
        return Err(ContractError::AccountCannotCreateAccount {});
    }

    let mut msgs = vec![];
    let mut wasm_msgs = vec![];
    let mut submsgs = vec![];
    let mut attrs = vec![];

    let current_job_id = STATE.load(deps.storage)?.current_job_id;

    // create job account, always instantiate a new Warp account
    if data.is_job_account.unwrap_or(false) {
        submsgs.push(SubMsg {
            id: REPLY_ID_CREATE_JOB_ACCOUNT_AND_JOB,
            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: Some(env.contract.address.to_string()),
                code_id: config.warp_account_code_id.u64(),
                msg: to_binary(&account::InstantiateMsg {
                    owner: info.sender.to_string(),
                    funds: data.funds,
                    job_id: Some(current_job_id),
                    is_job_account: Some(true),
                })?,
                funds: info.funds,
                label: format!("{} job account for {}", info.sender.to_string(), data.name),
            }),
            gas_limit: None,
            reply_on: ReplyOn::Always,
        });
        attrs.push(Attribute::new("action", "create_job_account"));
    }
    // create regular account
    else {
        // account already exists, send funds to account
        if ACCOUNTS().has(deps.storage, info.sender.clone()) {
            let account = ACCOUNTS().load(deps.storage, info.sender.clone())?;

            let cw_funds = match data.funds {
                None => {
                    vec![]
                }
                Some(funds) => funds,
            };

            if !info.funds.is_empty() {
                msgs.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: account.account.to_string(),
                    amount: info.funds.clone(),
                }))
            }

            for cw_fund in &cw_funds {
                msgs.push(CosmosMsg::Wasm(match cw_fund {
                    Fund::Cw20(cw20_fund) => WasmMsg::Execute {
                        contract_addr: deps
                            .api
                            .addr_validate(&cw20_fund.contract_addr)?
                            .to_string(),
                        msg: to_binary(&FundTransferMsgs::TransferFrom(TransferFromMsg {
                            owner: info.sender.clone().to_string(),
                            recipient: account.account.clone().to_string(),
                            amount: cw20_fund.amount,
                        }))?,
                        funds: vec![],
                    },
                    Fund::Cw721(cw721_fund) => WasmMsg::Execute {
                        contract_addr: deps
                            .api
                            .addr_validate(&cw721_fund.contract_addr)?
                            .to_string(),
                        msg: to_binary(&FundTransferMsgs::TransferNft(TransferNftMsg {
                            recipient: account.account.clone().to_string(),
                            token_id: cw721_fund.token_id.clone(),
                        }))?,
                        funds: vec![],
                    },
                }))
            }

            attrs.push(Attribute::new("action", "create_account"));
            attrs.push(Attribute::new("owner", account.owner));
            attrs.push(Attribute::new("account_address", account.account.clone()));

            //assume reward.amount == warp token allowance
            let fee =
                data.reward * Uint128::from(config.creation_fee_percentage) / Uint128::new(100);

            wasm_msgs = vec![
                WasmMsg::Execute {
                    contract_addr: account.account.to_string(),
                    msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {
                        msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                            to_address: env.contract.address.to_string(),
                            amount: vec![Coin::new((data.reward).u128(), config.fee_denom.clone())],
                        })],
                    }))?,
                    funds: vec![],
                },
                WasmMsg::Execute {
                    contract_addr: account.account.to_string(),
                    msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {
                        msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                            to_address: config.fee_collector.to_string(),
                            amount: vec![Coin::new((fee).u128(), config.fee_denom)],
                        })],
                    }))?,
                    funds: vec![],
                },
            ];
        }
        // account does not exist, create account
        else {
            submsgs.push(SubMsg {
                id: REPLY_ID_CREATE_ACCOUNT_AND_JOB,
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    admin: Some(env.contract.address.to_string()),
                    code_id: config.warp_account_code_id.u64(),
                    msg: to_binary(&account::InstantiateMsg {
                        owner: info.sender.to_string(),
                        funds: data.funds,
                        job_id: Some(current_job_id),
                        is_job_account: None,
                    })?,
                    funds: info.funds,
                    label: info.sender.to_string(),
                }),
                gas_limit: None,
                reply_on: ReplyOn::Always,
            });
            attrs.push(Attribute::new("action", "create_account"));
        }
    }

    let state = STATE.load(deps.storage)?;

    let job = PENDING_JOBS().update(deps.storage, current_job_id.u64(), |s| match s {
        None => Ok(Job {
            id: current_job_id,
            owner: info.sender,
            last_update_time: Uint64::from(env.block.time.seconds()),
            name: data.name,
            status: JobStatus::Pending,
            condition: data.condition.clone(),
            terminate_condition: data.terminate_condition.clone(),
            recurring: data.recurring,
            requeue_on_evict: data.requeue_on_evict,
            vars: data.vars,
            msgs: data.msgs,
            reward: data.reward,
            description: data.description,
            labels: data.labels,
            assets_to_withdraw: data.assets_to_withdraw.unwrap_or(vec![]),
            job_account: None,
        }),
        Some(_) => Err(ContractError::JobAlreadyExists {}),
    })?;
    attrs.push(Attribute::new("job_id", job.id));

    STATE.save(
        deps.storage,
        &State {
            current_job_id: state.current_job_id.checked_add(Uint64::new(1))?,
            q: state.q.checked_add(Uint64::new(1))?,
        },
    )?;

    Ok(Response::new()
        .add_attributes(attrs)
        .add_messages(msgs)
        .add_messages(wasm_msgs)
        .add_submessages(submsgs))
}

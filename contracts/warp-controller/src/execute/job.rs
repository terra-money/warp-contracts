use crate::contract::{
    REPLY_ID_CREATE_ACCOUNT_AND_SUB_ACCOUNT_AND_JOB, REPLY_ID_CREATE_SUB_ACCOUNT_AND_JOB,
    REPLY_ID_EXECUTE_JOB,
};
use crate::state::{JobQueue, ACCOUNTS, CONFIG, STATE};
use crate::ContractError;
use crate::ContractError::EvictionPeriodNotElapsed;
use account::{FirstFreeSubAccountResponse, GenericMsg};
use controller::account::{Fund, FundTransferMsgs, TransferFromMsg, TransferNftMsg};
use controller::job::{
    CreateJobMsg, DeleteJobMsg, EvictJobMsg, ExecuteJobMsg, Job, JobStatus, UpdateJobMsg,
};
use cosmwasm_std::{
    to_binary, Attribute, BalanceResponse, BankMsg, BankQuery, Coin, CosmosMsg, DepsMut, Env,
    MessageInfo, QueryRequest, ReplyOn, Response, StdResult, SubMsg, Uint128, Uint64, WasmMsg,
};
use resolver::QueryHydrateMsgsMsg;

const MAX_TEXT_LENGTH: usize = 280;

pub fn create_job(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: CreateJobMsg,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    if data.name.len() > MAX_TEXT_LENGTH {
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
    )?;

    // First try to query main account by account address (query index key which is account by sender)
    // This can happen when account contract calls controller's create_job
    // The result would be none if user (account owner) calls create_job directly
    let main_account = match ACCOUNTS()
        .idx
        .account
        .item(deps.storage, info.sender.clone())?
    {
        // create_job is called by account contract
        Some(record) => Some(record.1),
        // create_job is called by user
        None => match ACCOUNTS().may_load(deps.storage, info.sender.clone())? {
            // User has main account
            Some(account) => Some(account),
            // User does not have main account
            None => None,
        },
    };

    match main_account {
        None => {
            let create_main_account_submsg = SubMsg {
                id: REPLY_ID_CREATE_ACCOUNT_AND_SUB_ACCOUNT_AND_JOB,
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    admin: Some(env.contract.address.to_string()),
                    code_id: config.warp_account_code_id.u64(),
                    msg: to_binary(&account::InstantiateMsg {
                        owner: info.sender.to_string(),
                        funds: data.funds,
                        msgs: data.account_msgs,
                        is_sub_account: false,
                        main_account_addr: None,
                    })?,
                    funds: info.funds,
                    label: info.sender.to_string(),
                }),
                gas_limit: None,
                reply_on: ReplyOn::Always,
            };

            Ok(Response::new()
                .add_submessage(create_main_account_submsg)
                .add_attribute(
                    "action",
                    "create_job_and_new_main_account_and_new_sub_account",
                ))
        }
        Some(main_account) => {
            if main_account.owner != info.sender {
                return Err(ContractError::Unauthorized {});
            }
            let main_account_addr = main_account.account;
            let available_sub_account: FirstFreeSubAccountResponse =
                deps.querier.query_wasm_smart(
                    main_account_addr.clone(),
                    &account::QueryMsg::QueryFirstFreeSubAccount(
                        account::QueryFirstFreeSubAccountMsg {},
                    ),
                )?;
            match available_sub_account.sub_account {
                None => {
                    let create_sub_account_submsg = SubMsg {
                        id: REPLY_ID_CREATE_SUB_ACCOUNT_AND_JOB,
                        msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                            admin: Some(env.contract.address.to_string()),
                            code_id: config.warp_account_code_id.u64(),
                            msg: to_binary(&account::InstantiateMsg {
                                owner: info.sender.to_string(),
                                funds: data.funds,
                                msgs: data.account_msgs,
                                is_sub_account: true,
                                main_account_addr: Some(main_account_addr.clone().to_string()),
                            })?,
                            funds: info.funds,
                            label: info.sender.to_string(),
                        }),
                        gas_limit: None,
                        reply_on: ReplyOn::Always,
                    };

                    Ok(Response::new()
                        .add_submessage(create_sub_account_submsg)
                        .add_attribute("action", "create_job_and_new_sub_account"))
                }
                Some(sub_account) => {
                    let sub_account_addr = sub_account.account_addr;
                    let job = JobQueue::add(
                        &mut deps,
                        Job {
                            id: state.current_job_id,
                            prev_id: None,
                            owner: info.sender.clone(),
                            account: sub_account_addr.clone(),
                            last_update_time: Uint64::from(env.block.time.seconds()),
                            name: data.name,
                            status: JobStatus::Pending,
                            condition: data.condition.clone(),
                            terminate_condition: data.terminate_condition,
                            recurring: data.recurring,
                            requeue_on_evict: data.requeue_on_evict,
                            vars: data.vars,
                            msgs: data.msgs,
                            reward: data.reward,
                            description: data.description,
                            labels: data.labels,
                            assets_to_withdraw: data.assets_to_withdraw.unwrap_or(vec![]),
                        },
                    )?;

                    // Assume reward.amount == warp token allowance
                    let fee = data.reward * Uint128::from(config.creation_fee_percentage)
                        / Uint128::new(100);

                    let cw_funds_vec = match data.funds {
                        None => {
                            vec![]
                        }
                        Some(funds) => funds,
                    };

                    let mut fund_account_msgs: Vec<CosmosMsg> = vec![];

                    if !info.funds.is_empty() {
                        fund_account_msgs.push(CosmosMsg::Bank(BankMsg::Send {
                            to_address: sub_account_addr.clone().to_string(),
                            amount: info.funds.clone(),
                        }))
                    }

                    for cw_fund in &cw_funds_vec {
                        fund_account_msgs.push(CosmosMsg::Wasm(match cw_fund {
                            Fund::Cw20(cw20_fund) => WasmMsg::Execute {
                                contract_addr: deps
                                    .api
                                    .addr_validate(&cw20_fund.contract_addr)?
                                    .to_string(),
                                msg: to_binary(&FundTransferMsgs::TransferFrom(TransferFromMsg {
                                    owner: info.sender.clone().to_string(),
                                    recipient: sub_account_addr.clone().to_string(),
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
                                    recipient: sub_account_addr.clone().to_string(),
                                    token_id: cw721_fund.token_id.clone(),
                                }))?,
                                funds: vec![],
                            },
                        }))
                    }

                    let reward_send_msgs = vec![
                        // Job sends reward to controller
                        WasmMsg::Execute {
                            contract_addr: sub_account_addr.to_string(),
                            msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {
                                msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                                    to_address: env.contract.address.to_string(),
                                    amount: vec![Coin::new(
                                        (data.reward).u128(),
                                        config.fee_denom.clone(),
                                    )],
                                })],
                            }))?,
                            funds: vec![],
                        },
                        // Job owner sends fee to fee collector
                        WasmMsg::Execute {
                            contract_addr: sub_account_addr.to_string(),
                            msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {
                                msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                                    to_address: config.fee_collector.to_string(),
                                    amount: vec![Coin::new((fee).u128(), config.fee_denom)],
                                })],
                            }))?,
                            funds: vec![],
                        },
                    ];

                    let mut account_msgs: Vec<WasmMsg> = vec![];

                    if let Some(msgs) = data.account_msgs {
                        account_msgs = vec![WasmMsg::Execute {
                            contract_addr: sub_account_addr.to_string(),
                            msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg { msgs }))?,
                            funds: vec![],
                        }];
                    }

                    Ok(Response::new()
                        .add_messages(fund_account_msgs)
                        .add_messages(reward_send_msgs)
                        .add_messages(account_msgs)
                        .add_attribute("action", "create_job")
                        .add_attribute("job_id", job.id)
                        .add_attribute("job_owner", job.owner)
                        .add_attribute("job_name", job.name)
                        .add_attribute("job_status", serde_json_wasm::to_string(&job.status)?)
                        .add_attribute("job_condition", serde_json_wasm::to_string(&job.condition)?)
                        .add_attribute("job_msgs", serde_json_wasm::to_string(&job.msgs)?)
                        .add_attribute("job_reward", job.reward)
                        .add_attribute("job_creation_fee", fee)
                        .add_attribute("job_last_updated_time", job.last_update_time))
                }
            }
        }
    }
}

pub fn delete_job(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: DeleteJobMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let job = JobQueue::get(&deps, data.id.into())?;

    if job.status != JobStatus::Pending {
        return Err(ContractError::JobNotActive {});
    }

    if job.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let _new_job = JobQueue::finalize(&mut deps, env, job.id.into(), JobStatus::Cancelled)?;

    let fee = job.reward * Uint128::from(config.cancellation_fee_percentage) / Uint128::new(100);

    let cw20_send_msgs = vec![
        // Job owner sends reward minus fee back to account
        BankMsg::Send {
            to_address: job.account.to_string(),
            amount: vec![Coin::new(
                (job.reward - fee).u128(),
                config.fee_denom.clone(),
            )],
        },
        // Job owner sends fee to fee collector
        BankMsg::Send {
            to_address: config.fee_collector.to_string(),
            amount: vec![Coin::new(fee.u128(), config.fee_denom)],
        },
    ];

    Ok(Response::new()
        .add_messages(cw20_send_msgs)
        .add_attribute("action", "delete_job")
        .add_attribute("job_id", job.id)
        .add_attribute("job_status", serde_json_wasm::to_string(&job.status)?)
        .add_attribute("deletion_fee", fee))
}

pub fn update_job(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: UpdateJobMsg,
) -> Result<Response, ContractError> {
    let job = JobQueue::get(&deps, data.id.into())?;
    let config = CONFIG.load(deps.storage)?;

    if info.sender != job.owner {
        return Err(ContractError::Unauthorized {});
    }

    let added_reward = data.added_reward.unwrap_or(Uint128::new(0));

    if data.name.is_some() && data.name.clone().unwrap().len() > MAX_TEXT_LENGTH {
        return Err(ContractError::NameTooLong {});
    }

    if data.name.is_some() && data.name.clone().unwrap().is_empty() {
        return Err(ContractError::NameTooShort {});
    }

    let job = JobQueue::update(&mut deps, env.clone(), data)?;

    let fee = added_reward * Uint128::from(config.creation_fee_percentage) / Uint128::new(100);

    if !added_reward.is_zero() && fee.is_zero() {
        return Err(ContractError::RewardTooSmall {});
    }

    let mut cw20_send_msgs = vec![];

    if added_reward.u128() > 0 {
        cw20_send_msgs.push(
            // Job owner sends additional reward to controller
            WasmMsg::Execute {
                contract_addr: job.account.to_string(),
                msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {
                    msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                        to_address: env.contract.address.to_string(),
                        amount: vec![Coin::new((added_reward).u128(), config.fee_denom.clone())],
                    })],
                }))?,
                funds: vec![],
            },
        );
        cw20_send_msgs.push(
            // Job owner sends fee to fee collector
            WasmMsg::Execute {
                contract_addr: job.account.to_string(),
                msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {
                    msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                        to_address: config.fee_collector.to_string(),
                        amount: vec![Coin::new((fee).u128(), config.fee_denom)],
                    })],
                }))?,
                funds: vec![],
            },
        );
    }

    Ok(Response::new()
        .add_messages(cw20_send_msgs)
        .add_attribute("action", "update_job")
        .add_attribute("job_id", job.id)
        .add_attribute("job_owner", job.owner)
        .add_attribute("job_name", job.name)
        .add_attribute("job_status", serde_json_wasm::to_string(&job.status)?)
        .add_attribute("job_condition", serde_json_wasm::to_string(&job.condition)?)
        .add_attribute("job_msgs", serde_json_wasm::to_string(&job.msgs)?)
        .add_attribute("job_reward", job.reward)
        .add_attribute("job_update_fee", fee)
        .add_attribute("job_last_updated_time", job.last_update_time))
}

pub fn execute_job(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: ExecuteJobMsg,
) -> Result<Response, ContractError> {
    let _config = CONFIG.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let job = JobQueue::get(&deps, data.id.into())?;

    if job.status != JobStatus::Pending {
        return Err(ContractError::JobNotActive {});
    }

    let vars: String = deps.querier.query_wasm_smart(
        config.resolver_address.clone(),
        &resolver::QueryMsg::QueryHydrateVars(resolver::QueryHydrateVarsMsg {
            vars: job.vars,
            external_inputs: data.external_inputs,
            warp_account_addr: Some(job.account.to_string()),
        }),
    )?;

    let resolution: StdResult<bool> = deps.querier.query_wasm_smart(
        config.resolver_address.clone(),
        &resolver::QueryMsg::QueryResolveCondition(resolver::QueryResolveConditionMsg {
            condition: job.condition,
            vars: vars.clone(),
            warp_account_addr: Some(job.account.to_string()),
        }),
    );

    let mut attrs = vec![];
    let mut submsgs = vec![];

    if let Err(e) = resolution {
        attrs.push(Attribute::new("job_condition_status", "invalid"));
        attrs.push(Attribute::new("error", e.to_string()));
        JobQueue::finalize(&mut deps, env, job.id.into(), JobStatus::Failed)?;
    } else {
        attrs.push(Attribute::new("job_condition_status", "valid"));
        if !resolution? {
            return Ok(Response::new()
                .add_attribute("action", "execute_job")
                .add_attribute("condition", "false")
                .add_attribute("job_id", job.id));
        }

        submsgs.push(SubMsg {
            id: REPLY_ID_EXECUTE_JOB,
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: job.account.to_string(),
                msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {
                    msgs: deps.querier.query_wasm_smart(
                        config.resolver_address,
                        &resolver::QueryMsg::QueryHydrateMsgs(QueryHydrateMsgsMsg {
                            msgs: job.msgs,
                            vars,
                        }),
                    )?,
                }))?,
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Always,
        });
    }

    // Controller sends reward to executor
    let reward_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin::new(job.reward.u128(), config.fee_denom)],
    };

    Ok(Response::new()
        .add_submessages(submsgs)
        .add_message(reward_msg)
        .add_attribute("action", "execute_job")
        .add_attribute("executor", info.sender)
        .add_attribute("job_id", job.id)
        .add_attribute("job_reward", job.reward)
        .add_attributes(attrs))
}

pub fn evict_job(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: EvictJobMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    let job = JobQueue::get(&deps, data.id.into())?;

    let account_amount = deps
        .querier
        .query::<BalanceResponse>(&QueryRequest::Bank(BankQuery::Balance {
            address: job.account.to_string(),
            denom: config.fee_denom.clone(),
        }))?
        .amount
        .amount;

    if job.status != JobStatus::Pending {
        return Err(ContractError::Unauthorized {});
    }

    let t = if state.q < config.q_max {
        config.t_max - state.q * (config.t_max - config.t_min) / config.q_max
    } else {
        config.t_min
    };

    let a = if state.q < config.q_max {
        config.a_min
    } else {
        config.a_max
    };

    if env.block.time.seconds() - job.last_update_time.u64() < t.u64() {
        return Err(EvictionPeriodNotElapsed {});
    }

    let mut cosmos_msgs = vec![];

    let job_status;

    if job.requeue_on_evict && account_amount >= a {
        cosmos_msgs.push(
            // Controller sends reward to evictor
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: job.account.to_string(),
                msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {
                    msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                        to_address: info.sender.to_string(),
                        amount: vec![Coin::new(a.u128(), config.fee_denom)],
                    })],
                }))?,
                funds: vec![],
            }),
        );
        job_status = JobQueue::sync(&mut deps, env, job.clone())?.status;
    } else {
        job_status = JobQueue::finalize(&mut deps, env, job.id.into(), JobStatus::Evicted)?.status;

        cosmos_msgs.append(&mut vec![
            // Controller sends reward to evictor
            CosmosMsg::Bank(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![Coin::new(a.u128(), config.fee_denom.clone())],
            }),
            // Controller sends reward minus fee back to account
            CosmosMsg::Bank(BankMsg::Send {
                to_address: job.account.to_string(),
                amount: vec![Coin::new((job.reward - a).u128(), config.fee_denom)],
            }),
        ]);
    }

    Ok(Response::new()
        .add_attribute("action", "evict_job")
        .add_attribute("job_id", job.id)
        .add_attribute("job_status", serde_json_wasm::to_string(&job_status)?)
        .add_messages(cosmos_msgs))
}

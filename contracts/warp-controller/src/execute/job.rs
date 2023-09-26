use crate::state::{JobQueue, ACCOUNTS, CONFIG, STATE};
use crate::ContractError;
use crate::ContractError::EvictionPeriodNotElapsed;
use account::GenericMsg;
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

    let account_record = ACCOUNTS()
        .idx
        .account
        .item(deps.storage, info.sender.clone())?;

    let account = match account_record {
        None => ACCOUNTS()
            .load(deps.storage, info.sender)
            .map_err(|_e| ContractError::AccountDoesNotExist {})?,
        Some(record) => record.1,
    };

    let job = JobQueue::add(
        &mut deps,
        Job {
            id: state.current_job_id,
            prev_id: None,
            owner: account.owner,
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

    //assume reward.amount == warp token allowance
    let fee = data.reward * Uint128::from(config.creation_fee_percentage) / Uint128::new(100);

    let reward_send_msgs = vec![
        //send reward to controller
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

    let mut account_msgs: Vec<WasmMsg> = vec![];

    if let Some(msgs) = data.account_msgs {
        account_msgs = vec![WasmMsg::Execute {
            contract_addr: account.account.to_string(),
            msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg { msgs }))?,
            funds: vec![],
        }];
    }

    Ok(Response::new()
        .add_messages(reward_send_msgs)
        .add_attribute("action", "create_job")
        .add_attribute("job_id", job.id)
        .add_attribute("job_owner", job.owner)
        .add_attribute("job_name", job.name)
        .add_attribute("job_status", serde_json_wasm::to_string(&job.status)?)
        .add_attribute("job_condition", serde_json_wasm::to_string(&job.condition)?)
        .add_attribute("job_msgs", serde_json_wasm::to_string(&job.msgs)?)
        .add_attribute("job_reward", job.reward)
        .add_attribute("job_creation_fee", fee)
        .add_attribute("job_last_updated_time", job.last_update_time)
        .add_messages(account_msgs))
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

    let account = ACCOUNTS().load(deps.storage, info.sender)?;

    let _new_job = JobQueue::finalize(&mut deps, env, job.id.into(), JobStatus::Cancelled)?;

    let fee = job.reward * Uint128::from(config.cancellation_fee_percentage) / Uint128::new(100);

    let cw20_send_msgs = vec![
        //send reward minus fee back to account
        BankMsg::Send {
            to_address: account.account.to_string(),
            amount: vec![Coin::new(
                (job.reward - fee).u128(),
                config.fee_denom.clone(),
            )],
        },
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

    let account = ACCOUNTS().load(deps.storage, info.sender)?;

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
            //send reward to controller
            WasmMsg::Execute {
                contract_addr: account.account.to_string(),
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
            //send reward to controller
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
    let account = ACCOUNTS().load(deps.storage, job.owner.clone())?;

    if job.status != JobStatus::Pending {
        return Err(ContractError::JobNotActive {});
    }

    let vars: String = deps.querier.query_wasm_smart(
        config.resolver_address.clone(),
        &resolver::QueryMsg::QueryHydrateVars(resolver::QueryHydrateVarsMsg {
            vars: job.vars,
            external_inputs: data.external_inputs,
        }),
    )?;

    let resolution: StdResult<bool> = deps.querier.query_wasm_smart(
        config.resolver_address.clone(),
        &resolver::QueryMsg::QueryResolveCondition(resolver::QueryResolveConditionMsg {
            condition: job.condition,
            vars: vars.clone(),
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
            id: job.id.u64(),
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: account.account.to_string(),
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

    //send reward to executor
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
    let account = ACCOUNTS().load(deps.storage, job.owner.clone())?;

    let account_amount = deps
        .querier
        .query::<BalanceResponse>(&QueryRequest::Bank(BankQuery::Balance {
            address: account.account.to_string(),
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
            //send reward to evictor
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: account.account.to_string(),
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
        job_status =
            JobQueue::finalize(&mut deps, env, job.id.into(), JobStatus::Evicted)?.status;

        cosmos_msgs.append(&mut vec![
            //send reward minus fee back to account
            CosmosMsg::Bank(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![Coin::new(a.u128(), config.fee_denom.clone())],
            }),
            CosmosMsg::Bank(BankMsg::Send {
                to_address: account.account.to_string(),
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

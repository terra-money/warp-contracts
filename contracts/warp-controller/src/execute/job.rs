use crate::state::{ACCOUNTS, CONFIG, FINISHED_JOBS, PENDING_JOBS, STATE};
use crate::ContractError;
use crate::ContractError::EvictionPeriodNotElapsed;
use account::GenericMsg;
use controller::job::{
    CreateJobMsg, DeleteJobMsg, EvictJobMsg, ExecuteJobMsg, Job, JobStatus, UpdateJobMsg,
};
use controller::State;
use cosmwasm_std::{
    to_binary, Attribute, BalanceResponse, BankMsg, BankQuery, Coin, CosmosMsg, DepsMut, Env,
    MessageInfo, QueryRequest, ReplyOn, Response, StdResult, SubMsg, Uint128, Uint64, WasmMsg,
};
use resolver::QueryHydrateMsgsMsg;

const MAX_TEXT_LENGTH: usize = 280;

pub fn create_job(
    deps: DepsMut,
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

    let q = ACCOUNTS()
        .idx
        .account
        .item(deps.storage, info.sender.clone())?;

    let account = match q {
        None => ACCOUNTS()
            .load(deps.storage, info.sender)
            .map_err(|_e| ContractError::AccountDoesNotExist {})?,
        Some(q) => q.1,
    };

    // let mut msgs = vec![];
    // for msg in data.msgs {
    //     msgs.push(serde_json_wasm::from_str::<CosmosMsg>(msg.as_str())?)
    // }

    let job = PENDING_JOBS().update(deps.storage, state.current_job_id.u64(), |s| match s {
        None => Ok(Job {
            id: state.current_job_id,
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
        }),
        Some(_) => Err(ContractError::JobAlreadyExists {}),
    })?;

    STATE.save(
        deps.storage,
        &State {
            current_job_id: state.current_job_id.checked_add(Uint64::new(1))?,
            q: state.q.checked_add(Uint64::new(1))?,
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
        .add_attribute("job_last_updated_time", job.last_update_time))
}

pub fn delete_job(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data: DeleteJobMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    let job = PENDING_JOBS().load(deps.storage, data.id.u64())?;

    if job.status != JobStatus::Pending {
        return Err(ContractError::JobNotActive {});
    }

    if job.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let account = ACCOUNTS().load(deps.storage, info.sender)?;

    PENDING_JOBS().remove(deps.storage, data.id.u64())?;
    let _new_job = FINISHED_JOBS().update(deps.storage, data.id.u64(), |h| match h {
        None => Ok(Job {
            id: job.id,
            owner: job.owner,
            last_update_time: job.last_update_time,
            name: job.name,
            status: JobStatus::Cancelled,
            condition: job.condition,
            terminate_condition: job.terminate_condition,
            msgs: job.msgs,
            vars: job.vars,
            recurring: job.recurring,
            requeue_on_evict: job.requeue_on_evict,
            reward: job.reward,
            description: job.description,
            labels: job.labels,
            assets_to_withdraw: job.assets_to_withdraw,
        }),
        Some(_job) => Err(ContractError::JobAlreadyFinished {}),
    })?;

    STATE.save(
        deps.storage,
        &State {
            current_job_id: state.current_job_id,
            q: state.q.checked_sub(Uint64::new(1))?,
        },
    )?;

    let fee = job.reward * Uint128::from(config.cancellation_fee_percentage) / Uint128::new(100);

    let cw20_send_msgs = vec![
        //send reward minus fee back to account
        BankMsg::Send {
            to_address: account.account.to_string(),
            amount: vec![Coin::new((job.reward - fee).u128(), config.fee_denom.clone())],
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
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: UpdateJobMsg,
) -> Result<Response, ContractError> {
    let job = PENDING_JOBS().load(deps.storage, data.id.u64())?;
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

    let job = PENDING_JOBS().update(deps.storage, data.id.u64(), |h| match h {
        None => Err(ContractError::JobDoesNotExist {}),
        Some(job) => Ok(Job {
            id: job.id,
            owner: job.owner,
            last_update_time: if added_reward > config.minimum_reward {
                Uint64::new(env.block.time.seconds())
            } else {
                job.last_update_time
            },
            name: data.name.unwrap_or(job.name),
            description: data.description.unwrap_or(job.description),
            labels: data.labels.unwrap_or(job.labels),
            status: job.status,
            condition: job.condition,
            terminate_condition: job.terminate_condition,
            msgs: job.msgs,
            vars: job.vars,
            recurring: job.recurring,
            requeue_on_evict: job.requeue_on_evict,
            reward: job.reward + added_reward,
            assets_to_withdraw: job.assets_to_withdraw,
        }),
    })?;

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
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data: ExecuteJobMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let job = PENDING_JOBS().load(deps.storage, data.id.u64())?;
    let account = ACCOUNTS().load(deps.storage, job.owner.clone())?;

    if !ACCOUNTS().has(deps.storage, info.sender.clone()) {
        return Err(ContractError::AccountDoesNotExist {});
    }

    let keeper_account = ACCOUNTS().load(deps.storage, info.sender.clone())?;

    if job.status != JobStatus::Pending {
        return Err(ContractError::JobNotActive {});
    }

    // let vars = hydrate_vars(
    //     deps.as_ref(),
    //     env.clone(),
    //     job.vars.clone(),
    //     data.external_inputs,
    // )?;

    let vars: String = deps.querier.query_wasm_smart(
        config.resolver_address.clone(),
        &resolver::QueryMsg::QueryHydrateVars(resolver::QueryHydrateVarsMsg {
            vars: job.vars,
            external_inputs: data.external_inputs,
        }),
    )?;

    //
    // let resolution = resolve_cond(deps.as_ref(), env, job.condition.clone(), &vars);

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
        let job = PENDING_JOBS().load(deps.storage, data.id.u64())?;
        FINISHED_JOBS().save(
            deps.storage,
            data.id.u64(),
            &Job {
                id: job.id,
                owner: job.owner,
                last_update_time: job.last_update_time,
                name: job.name,
                description: job.description,
                labels: job.labels,
                status: JobStatus::Failed,
                condition: job.condition,
                terminate_condition: job.terminate_condition,
                msgs: job.msgs,
                vars,
                recurring: job.recurring,
                requeue_on_evict: job.requeue_on_evict,
                reward: job.reward,
                assets_to_withdraw: job.assets_to_withdraw,
            },
        )?;
        PENDING_JOBS().remove(deps.storage, data.id.u64())?;
        STATE.save(
            deps.storage,
            &State {
                current_job_id: state.current_job_id,
                q: state.q.checked_sub(Uint64::new(1))?,
            },
        )?;
    } else {
        attrs.push(Attribute::new("job_condition_status", "valid"));
        if !resolution? {
            return Err(ContractError::JobNotActive {});
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

    let reward_msg = BankMsg::Send {
        to_address: keeper_account.account.to_string(),
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
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: EvictJobMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    let job = PENDING_JOBS().load(deps.storage, data.id.u64())?;
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
        job_status = PENDING_JOBS()
            .update(deps.storage, data.id.u64(), |j| match j {
                None => Err(ContractError::JobDoesNotExist {}),
                Some(job) => Ok(Job {
                    id: job.id,
                    owner: job.owner,
                    last_update_time: Uint64::new(env.block.time.seconds()),
                    name: job.name,
                    description: job.description,
                    labels: job.labels,
                    status: JobStatus::Pending,
                    condition: job.condition,
                    terminate_condition: job.terminate_condition,
                    msgs: job.msgs,
                    vars: job.vars,
                    recurring: job.recurring,
                    requeue_on_evict: job.requeue_on_evict,
                    reward: job.reward,
                    assets_to_withdraw: job.assets_to_withdraw,
                }),
            })?
            .status;
    } else {
        PENDING_JOBS().remove(deps.storage, data.id.u64())?;
        job_status = FINISHED_JOBS()
            .update(deps.storage, data.id.u64(), |j| match j {
                None => Ok(Job {
                    id: job.id,
                    owner: job.owner,
                    last_update_time: Uint64::new(env.block.time.seconds()),
                    name: job.name,
                    description: job.description,
                    labels: job.labels,
                    status: JobStatus::Evicted,
                    condition: job.condition,
                    terminate_condition: job.terminate_condition,
                    msgs: job.msgs,
                    vars: job.vars,
                    recurring: job.recurring,
                    requeue_on_evict: job.requeue_on_evict,
                    reward: job.reward,
                    assets_to_withdraw: job.assets_to_withdraw,
                }),
                Some(_) => Err(ContractError::JobAlreadyExists {}),
            })?
            .status;

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

        STATE.save(deps.storage, &State {
            current_job_id: state.current_job_id,
            q: state.q.checked_sub(Uint64::new(1))?,
        })?;
    }

    Ok(Response::new()
        .add_attribute("action", "evict_job")
        .add_attribute("job_id", job.id)
        .add_attribute("job_status", serde_json_wasm::to_string(&job_status)?)
        .add_messages(cosmos_msgs))
}

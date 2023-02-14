use crate::state::{ACCOUNTS, CONFIG, FINISHED_JOBS, PENDING_JOBS, STATE};
use crate::util::condition::resolve_cond;
use crate::util::variable::{all_vector_vars_present, has_duplicates, hydrate_msgs, hydrate_vars, msgs_valid, string_vars_in_vector, vars_valid};
use crate::ContractError;
use crate::ContractError::EvictionPeriodNotElapsed;
use cosmwasm_std::{
    to_binary, Attribute, BalanceResponse, BankMsg, BankQuery, Coin, CosmosMsg, DepsMut, Env,
    MessageInfo, QueryRequest, ReplyOn, Response, SubMsg, Uint128, Uint64, WasmMsg,
};
use warp_protocol::controller::job::{
    CreateJobMsg, DeleteJobMsg, EvictJobMsg, ExecuteJobMsg, Job, JobStatus, UpdateJobMsg,
};
use warp_protocol::controller::State;

pub fn create_job(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: CreateJobMsg,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    if data.name.len() > 140 {
        return Err(ContractError::NameTooLong {});
    }

    if data.name.is_empty() {
        return Err(ContractError::NameTooShort {});
    }

    if data.reward < config.minimum_reward || data.reward.is_zero() {
        return Err(ContractError::RewardTooSmall {});
    }

    if !vars_valid(&data.vars) {
        return Err(ContractError::InvalidVariables {});
    }

    if has_duplicates(&data.vars) {
        return Err(ContractError::VariablesContainDuplicates {});
    }

    let cond_string = serde_json_wasm::to_string(&data.condition)?;
    let msg_string = serde_json_wasm::to_string(&data.msgs)?;

    if !(string_vars_in_vector(&data.vars, &cond_string)
        && string_vars_in_vector(&data.vars, &msg_string))
    {
        return Err(ContractError::VariablesMissingFromVector {});
    }

    if !all_vector_vars_present(&data.vars, format!("{}{}", cond_string, msg_string)) {
        return Err(ContractError::ExcessVariablesInVector {});
    }
    
    if !msgs_valid(&data.msgs, &data.vars)? {
        return Err(ContractError::MsgError { msg: "msgs are invalid".to_string() });
    }

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
            recurring: data.recurring,
            requeue_on_evict: data.requeue_on_evict,
            vars: data.vars,
            msgs: data.msgs,
            reward: data.reward,
        }),
        Some(_) => Err(ContractError::JobAlreadyExists {}),
    })?;

    STATE.save(
        deps.storage,
        &State {
            current_job_id: state.current_job_id.checked_add(Uint64::new(1))?,
            current_template_id: state.current_template_id,
            q: state.q.checked_add(Uint64::new(1))?,
        },
    )?;

    //assume reward.amount == warp token allowance
    let fee = data.reward * Uint128::from(config.creation_fee_percentage) / Uint128::new(100);

    let reward_send_msgs = vec![
        //send reward to controller
        WasmMsg::Execute {
            contract_addr: account.account.to_string(),
            msg: to_binary(&warp_protocol::account::ExecuteMsg {
                msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                    to_address: env.contract.address.to_string(),
                    amount: vec![Coin::new((data.reward + fee).u128(), "uluna")],
                })],
            })?,
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
            msgs: job.msgs,
            vars: job.vars,
            recurring: job.recurring,
            requeue_on_evict: job.requeue_on_evict,
            reward: job.reward,
        }),
        Some(_job) => Err(ContractError::JobAlreadyFinished {}),
    })?;

    STATE.save(
        deps.storage,
        &State {
            current_job_id: state.current_job_id,
            current_template_id: state.current_template_id,
            q: state.q.checked_sub(Uint64::new(1))?,
        },
    )?;

    let fee = job.reward * Uint128::from(config.cancellation_fee_percentage) / Uint128::new(100);

    let cw20_send_msgs = vec![
        //send reward minus fee back to account
        BankMsg::Send {
            to_address: account.account.to_string(),
            amount: vec![Coin::new((job.reward - fee).u128(), "uluna")],
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

    if data.name.is_some() && data.name.clone().unwrap().len() > 140 {
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
            last_update_time: if !added_reward.is_zero() {
                Uint64::new(env.block.time.seconds())
            } else {
                job.last_update_time
            },
            name: data.name.unwrap_or(job.name),
            status: job.status,
            condition: job.condition,
            msgs: job.msgs,
            vars: job.vars,
            recurring: job.recurring,
            requeue_on_evict: job.requeue_on_evict,
            reward: job.reward + added_reward,
        }),
    })?;

    //todo: sanitize updates

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
                msg: to_binary(&warp_protocol::account::ExecuteMsg {
                    msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                        to_address: env.contract.address.to_string(),
                        amount: vec![Coin::new((added_reward + fee).u128(), "uluna")],
                    })],
                })?,
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
    env: Env,
    info: MessageInfo,
    data: ExecuteJobMsg,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    let job = PENDING_JOBS().load(deps.storage, data.id.u64())?;
    let account = ACCOUNTS().load(deps.storage, job.owner.clone())?;

    if !ACCOUNTS().has(deps.storage, info.sender.clone()) {
        return Err(ContractError::AccountDoesNotExist {});
    }

    let keeper_account = ACCOUNTS().load(deps.storage, info.sender.clone())?;

    if job.status != JobStatus::Pending {
        return Err(ContractError::JobNotActive {});
    }

    let vars = hydrate_vars(
        deps.as_ref(),
        env.clone(),
        job.vars.clone(),
        data.external_inputs,
    )?;

    let resolution = resolve_cond(deps.as_ref(), env, job.condition.clone(), &vars);

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
                status: JobStatus::Failed,
                condition: job.condition,
                msgs: job.msgs,
                vars,
                recurring: job.recurring,
                requeue_on_evict: job.requeue_on_evict,
                reward: job.reward,
            },
        )?;
        PENDING_JOBS().remove(deps.storage, data.id.u64())?;
        STATE.save(
            deps.storage,
            &State {
                current_job_id: state.current_job_id,
                current_template_id: state.current_template_id,
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
                msg: to_binary(&warp_protocol::account::ExecuteMsg {
                    msgs: hydrate_msgs(job.msgs.clone(), vars)?,
                })?,
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Always,
        });
    }

    let reward_msg = BankMsg::Send {
        to_address: keeper_account.account.to_string(),
        amount: vec![Coin::new(job.reward.u128(), "uluna")],
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
            denom: "uluna".to_string(),
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
            //send reward to controller
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: account.account.to_string(),
                msg: to_binary(&warp_protocol::account::ExecuteMsg {
                    msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                        to_address: info.sender.to_string(),
                        amount: vec![Coin::new(a.u128(), "uluna")],
                    })],
                })?,
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
                    status: JobStatus::Pending,
                    condition: job.condition,
                    msgs: job.msgs,
                    vars: job.vars,
                    recurring: job.recurring,
                    requeue_on_evict: job.requeue_on_evict,
                    reward: job.reward,
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
                    status: JobStatus::Evicted,
                    condition: job.condition,
                    msgs: job.msgs,
                    vars: job.vars,
                    recurring: job.recurring,
                    requeue_on_evict: job.requeue_on_evict,
                    reward: job.reward,
                }),
                Some(_) => Err(ContractError::JobAlreadyExists {}),
            })?
            .status;

        cosmos_msgs.append(&mut vec![
            //send reward minus fee back to account
            CosmosMsg::Bank(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![Coin::new(a.u128(), "uluna")],
            }),
            CosmosMsg::Bank(BankMsg::Send {
                to_address: account.account.to_string(),
                amount: vec![Coin::new((job.reward - a).u128(), "uluna")],
            }),
        ]);
    }

    Ok(Response::new()
        .add_attribute("action", "evict_job")
        .add_attribute("job_id", job.id)
        .add_attribute("job_status", serde_json_wasm::to_string(&job_status)?)
        .add_messages(cosmos_msgs))
}

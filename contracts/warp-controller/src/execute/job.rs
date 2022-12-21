use crate::state::{ACCOUNTS, CONFIG, FINISHED_JOBS, PENDING_JOBS, STATE};
use crate::util::condition::resolve_cond;
use crate::ContractError;
use cosmwasm_std::{
    to_binary, Attribute, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, ReplyOn, Response,
    SubMsg, Uint128, Uint64, WasmMsg,
};
use warp_protocol::controller::controller::State;
use warp_protocol::controller::job::{
    CreateJobMsg, DeleteJobMsg, ExecuteJobMsg, Job, JobStatus, UpdateJobMsg,
};

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

    if data.name.len() < 1 {
        return Err(ContractError::NameTooShort {});
    }

    if data.reward < config.minimum_reward || data.reward.is_zero() {
        return Err(ContractError::RewardTooSmall {});
    }

    let q = ACCOUNTS()
        .idx
        .account
        .item(deps.storage, info.sender.clone())?;

    let account = match q {
        None => ACCOUNTS()
            .load(deps.storage, info.sender.clone())
            .map_err(|_e| ContractError::AccountDoesNotExist {})?,
        Some(q) => q.1,
    };

    let mut msgs = vec![];
    for msg in data.msgs {
        msgs.push(serde_json_wasm::from_str::<CosmosMsg>(msg.as_str())?)
    }

    let job = PENDING_JOBS().update(deps.storage, state.current_job_id.u64(), |s| match s {
        None => Ok(Job {
            id: state.current_job_id,
            owner: account.owner,
            last_update_time: Uint64::from(env.block.time.seconds()),
            name: data.name,
            status: JobStatus::Pending,
            condition: data.condition.clone(),
            msgs,
            reward: data.reward.clone(),
        }),
        Some(_) => Err(ContractError::JobAlreadyExists {}),
    })?;

    STATE.save(
        deps.storage,
        &State {
            current_job_id: state.current_job_id.saturating_add(Uint64::new(1)),
            current_template_id: state.current_template_id,
        },
    )?;

    //assume reward.amount == warp token allowance
    let fee = data.reward * config.creation_fee_percentage / Uint128::new(100);

    let reward_send_msgs = vec![
        //send reward to controller
        WasmMsg::Execute {
            contract_addr: account.account.to_string(),
            msg: to_binary(&warp_protocol::account::account::ExecuteMsg {
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
    let job = PENDING_JOBS().load(deps.storage, data.id.u64())?;

    if job.status != JobStatus::Pending {
        return Err(ContractError::JobNotActive {});
    }

    if job.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let account = ACCOUNTS().load(deps.storage, info.sender.clone())?;

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
            reward: job.reward.clone(),
        }),
        Some(_job) => Err(ContractError::JobAlreadyFinished {}),
    })?;

    let fee = job.reward * config.cancellation_fee_percentage / Uint128::new(100);

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

    let account = ACCOUNTS().load(deps.storage, info.sender.clone())?;

    let added_reward = data.added_reward.unwrap_or(Uint128::new(0));

    if data.name.is_some() && data.name.clone().unwrap().len() > 140 {
        return Err(ContractError::NameTooLong {});
    }

    if data.name.is_some() && data.name.clone().unwrap().len() < 1 {
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
            reward: job.reward + added_reward,
        }),
    })?;

    //todo: sanitize updates

    //assume reward.amount == warp token allowance
    let fee = added_reward * config.creation_fee_percentage / Uint128::new(100);

    if !added_reward.is_zero() && fee.is_zero() {
        return Err(ContractError::RewardTooSmall {});
    }

    let cw20_send_msgs = vec![
        //send reward to controller
        WasmMsg::Execute {
            contract_addr: account.account.to_string(),
            msg: to_binary(&warp_protocol::account::account::ExecuteMsg {
                msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                    to_address: env.contract.address.to_string(),
                    amount: vec![Coin::new((added_reward + fee).u128(), "uluna")],
                })],
            })?,
            funds: vec![],
        },
    ];

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
    let job = PENDING_JOBS().load(deps.storage, data.id.u64())?;
    let account = ACCOUNTS().load(deps.storage, job.owner.clone())?;

    if !ACCOUNTS().has(deps.storage, info.sender.clone()) {
        return Err(ContractError::AccountDoesNotExist {});
    }

    let keeper_account = ACCOUNTS().load(deps.storage, info.sender.clone())?;

    if job.status != JobStatus::Pending {
        return Err(ContractError::JobNotActive {});
    }

    let resolution = resolve_cond(deps.as_ref(), env.clone(), job.condition.clone());

    let mut attrs = vec![];

    let mut submsgs = vec![];

    if resolution.is_err() {
        attrs.push(Attribute::new("job_condition_status", "invalid"));
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
                reward: job.reward,
            },
        )?;
        PENDING_JOBS().remove(deps.storage, data.id.u64())?;
    } else {
        attrs.push(Attribute::new("job_condition_status", "valid"));
        if !resolution? {
            return Err(ContractError::JobNotActive {});
        }
        submsgs.push(SubMsg {
            id: job.id.u64(),
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: account.account.to_string(),
                msg: to_binary(&warp_protocol::account::account::ExecuteMsg {
                    msgs: job.msgs.clone(),
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
        .add_attribute("job_reward", job.reward)
        .add_attributes(attrs))
}

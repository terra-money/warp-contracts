use cosmwasm_std::{
    to_binary, Attribute, BalanceResponse, BankMsg, BankQuery, Coin, CosmosMsg, DepsMut, Env,
    MessageInfo, QueryRequest, ReplyOn, Response, StdResult, SubMsg, Uint128, Uint64, WasmMsg,
};

use crate::{
    contract::{
        REPLY_ID_CREATE_ACCOUNT_AND_JOB, REPLY_ID_CREATE_ACCOUNT_TRACKER_AND_ACCOUNT_AND_JOB,
        REPLY_ID_EXECUTE_JOB,
    },
    state::{JobQueue, JOB_ACCOUNT_TRACKERS, LEGACY_ACCOUNTS, STATE},
    util::{
        fee::deduct_reward_and_fee_from_native_funds,
        legacy_account::is_legacy_account,
        msg::{
            build_account_execute_generic_msgs, build_account_withdraw_assets_msg,
            build_free_account_msg, build_instantiate_warp_account_msg,
            build_instantiate_warp_job_account_tracker_msg, build_occupy_account_msg,
            build_transfer_cw20_msg, build_transfer_cw721_msg, build_transfer_native_funds_msg,
        },
    },
    ContractError,
};

use controller::{
    account::CwFund,
    job::{CreateJobMsg, DeleteJobMsg, EvictJobMsg, ExecuteJobMsg, Job, JobStatus, UpdateJobMsg},
    Config,
};
use job_account::GenericMsg;
use job_account_tracker::FirstFreeAccountResponse;
use resolver::QueryHydrateMsgsMsg;

const MAX_TEXT_LENGTH: usize = 280;

pub fn create_job(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: CreateJobMsg,
    config: Config,
    fee_denom_paid_amount: Uint128,
) -> Result<Response, ContractError> {
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

    let fee = data.reward * Uint128::from(config.creation_fee_percentage) / Uint128::new(100);
    let reward_plus_fee = data.reward + fee;
    if reward_plus_fee > fee_denom_paid_amount {
        return Err(ContractError::InsufficientFundsToPayForRewardAndFee {});
    }

    // Reward and fee will always be in native denom
    let native_funds_minus_reward_and_fee = deduct_reward_and_fee_from_native_funds(
        info.funds.clone(),
        config.fee_denom.clone(),
        reward_plus_fee,
    );

    let mut submsgs = vec![];
    let mut msgs = vec![];
    let mut attrs = vec![];

    // Job owner sends reward to controller when it calls create_job
    // Reward stays at controller, no need to send it elsewhere

    msgs.push(
        // Job owner sends fee to controller when it calls create_job
        // Controller sends fee to fee collector
        build_transfer_native_funds_msg(
            config.fee_collector.to_string(),
            vec![Coin::new(fee.u128(), config.fee_denom.clone())],
        ),
    );

    let job_account_tracker = JOB_ACCOUNT_TRACKERS.may_load(deps.storage, &info.sender)?;
    let state = STATE.load(deps.storage)?;
    let mut job = JobQueue::add(
        &mut deps,
        Job {
            id: state.current_job_id,
            prev_id: None,
            owner: info.sender.clone(),
            // Account uses a placeholder value for now, will update it to job account address if job account exists or after created
            // Update will happen either in create_job (exists free job account) or reply (after creation), so it's atomic
            // And we guarantee we do not read this value before it's updated
            account: info.sender.clone(),
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

    match job_account_tracker {
        None => {
            // Create account tracker then create account then create job in reply
            submsgs.push(SubMsg {
                id: REPLY_ID_CREATE_ACCOUNT_TRACKER_AND_ACCOUNT_AND_JOB,
                msg: build_instantiate_warp_job_account_tracker_msg(
                    env.contract.address.to_string(),
                    config.warp_job_account_tracker_code_id.u64(),
                    info.sender.to_string(),
                ),
                gas_limit: None,
                reply_on: ReplyOn::Always,
            });

            attrs.push(Attribute::new(
                "action",
                "create_job_account_tracker_and_account_and_job",
            ));
        }
        Some(job_account_tracker) => {
            let available_account: FirstFreeAccountResponse = deps.querier.query_wasm_smart(
                job_account_tracker.clone(),
                &job_account_tracker::QueryMsg::QueryFirstFreeAccount(
                    job_account_tracker::QueryFirstFreeAccountMsg {},
                ),
            )?;
            match available_account.account {
                None => {
                    // Create account then create job in reply
                    submsgs.push(SubMsg {
                        id: REPLY_ID_CREATE_ACCOUNT_AND_JOB,
                        msg: build_instantiate_warp_account_msg(
                            job.id,
                            env.contract.address.to_string(),
                            config.warp_account_code_id.u64(),
                            info.sender.to_string(),
                            job_account_tracker.clone().to_string(),
                            native_funds_minus_reward_and_fee,
                            data.cw_funds,
                            data.account_msgs,
                        ),
                        gas_limit: None,
                        reply_on: ReplyOn::Always,
                    });

                    attrs.push(Attribute::new("action", "create_account_and_job"));
                }
                Some(available_account) => {
                    let available_account_addr = available_account.addr;
                    // Update job.account from placeholder value to job account
                    job.account = available_account_addr.clone();
                    JobQueue::sync(&mut deps, env, job.clone())?;

                    if !native_funds_minus_reward_and_fee.is_empty() {
                        // Fund account in native coins
                        msgs.push(build_transfer_native_funds_msg(
                            available_account_addr.clone().to_string(),
                            native_funds_minus_reward_and_fee,
                        ))
                    }

                    if let Some(cw_funds) = data.cw_funds {
                        // Fund account in CW20 / CW721 tokens
                        for cw_fund in cw_funds {
                            msgs.push(match cw_fund {
                                CwFund::Cw20(cw20_fund) => build_transfer_cw20_msg(
                                    deps.api
                                        .addr_validate(&cw20_fund.contract_addr)?
                                        .to_string(),
                                    info.sender.clone().to_string(),
                                    available_account_addr.clone().to_string(),
                                    cw20_fund.amount,
                                ),
                                CwFund::Cw721(cw721_fund) => build_transfer_cw721_msg(
                                    deps.api
                                        .addr_validate(&cw721_fund.contract_addr)?
                                        .to_string(),
                                    available_account_addr.clone().to_string(),
                                    cw721_fund.token_id.clone(),
                                ),
                            })
                        }
                    }

                    if let Some(account_msgs) = data.account_msgs {
                        // Account execute msgs
                        msgs.push(build_account_execute_generic_msgs(
                            available_account_addr.to_string(),
                            account_msgs,
                        ));
                    }

                    // Occupy account
                    msgs.push(build_occupy_account_msg(
                        job_account_tracker.to_string(),
                        available_account_addr.to_string(),
                        job.id,
                    ));

                    attrs.push(Attribute::new("action", "create_job"));
                    attrs.push(Attribute::new("job_id", job.id));
                    attrs.push(Attribute::new("job_owner", job.owner));
                    attrs.push(Attribute::new("job_name", job.name));
                    attrs.push(Attribute::new(
                        "job_status",
                        serde_json_wasm::to_string(&job.status)?,
                    ));
                    attrs.push(Attribute::new(
                        "job_condition",
                        serde_json_wasm::to_string(&job.condition)?,
                    ));
                    attrs.push(Attribute::new(
                        "job_msgs",
                        serde_json_wasm::to_string(&job.msgs)?,
                    ));
                    attrs.push(Attribute::new("job_reward", job.reward));
                    attrs.push(Attribute::new("job_creation_fee", fee));
                    attrs.push(Attribute::new(
                        "job_last_updated_time",
                        job.last_update_time,
                    ));
                }
            }
        }
    }

    Ok(Response::new()
        .add_submessages(submsgs)
        .add_messages(msgs)
        .add_attributes(attrs))
}

pub fn delete_job(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: DeleteJobMsg,
    config: Config,
    fee_denom_paid_amount: Uint128,
) -> Result<Response, ContractError> {
    let job = JobQueue::get(&deps, data.id.into())?;
    let legacy_account = LEGACY_ACCOUNTS().may_load(deps.storage, job.owner.clone())?;
    let job_account_addr = job.account.clone();

    if job.status != JobStatus::Pending {
        return Err(ContractError::JobNotActive {});
    }

    if job.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let _new_job = JobQueue::finalize(&mut deps, env, job.id.into(), JobStatus::Cancelled)?;

    let fee = job.reward * Uint128::from(config.cancellation_fee_percentage) / Uint128::new(100);
    if fee > fee_denom_paid_amount {
        return Err(ContractError::InsufficientFundsToPayForFee {});
    }

    let mut msgs = vec![];

    // Controller sends reward minus cancellation fee back to job owner
    msgs.push(build_transfer_native_funds_msg(
        job.owner.to_string(),
        vec![Coin::new(
            (job.reward - fee).u128(),
            config.fee_denom.clone(),
        )],
    ));

    // Job owner sends fee to controller when it calls delete_job
    // Controller sends cancellation fee to fee collector
    msgs.push(build_transfer_native_funds_msg(
        config.fee_collector.to_string(),
        vec![Coin::new(fee.u128(), config.fee_denom)],
    ));

    if !is_legacy_account(legacy_account, job_account_addr.clone()) {
        // For job not using legacy account, job owner must already have account tracker instantiated
        let job_account_tracker = JOB_ACCOUNT_TRACKERS.load(deps.storage, &job.owner)?;
        // Free account
        msgs.push(build_free_account_msg(
            job_account_tracker.to_string(),
            job_account_addr.to_string(),
        ));
    }

    // Job owner withdraw all assets that are listed from warp account to itself
    msgs.push(build_account_withdraw_assets_msg(
        job_account_addr.clone().to_string(),
        job.assets_to_withdraw,
    ));

    Ok(Response::new()
        .add_messages(msgs)
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
    config: Config,
    fee_denom_paid_amount: Uint128,
) -> Result<Response, ContractError> {
    let job = JobQueue::get(&deps, data.id.into())?;

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
    if fee + added_reward > fee_denom_paid_amount {
        return Err(ContractError::InsufficientFundsToPayForRewardAndFee {});
    }

    let mut msgs = vec![];

    if added_reward > Uint128::zero() {
        // Job owner sends reward to controller when it calls create_job
        // Reward stays at controller, no need to send it elsewhere

        msgs.push(
            // Job owner sends fee to controller when it calls update_job
            // Controller sends update fee to fee collector
            WasmMsg::Execute {
                contract_addr: job.account.to_string(),
                msg: to_binary(&job_account::ExecuteMsg::Generic(GenericMsg {
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
        .add_messages(msgs)
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
    config: Config,
) -> Result<Response, ContractError> {
    let job = JobQueue::get(&deps, data.id.into())?;
    let legacy_account = LEGACY_ACCOUNTS().may_load(deps.storage, job.owner.clone())?;
    let job_account_addr = job.account.clone();

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
    let mut msgs = vec![];
    let mut submsgs = vec![];

    if let Err(e) = resolution {
        attrs.push(Attribute::new("job_condition_status", "invalid"));
        attrs.push(Attribute::new("error", e.to_string()));
        JobQueue::finalize(&mut deps, env, job.id.into(), JobStatus::Failed)?;

        // Withdraw reward to job owner
        msgs.push(build_account_withdraw_assets_msg(
            job.account.to_string(),
            job.assets_to_withdraw,
        ));
    } else {
        attrs.push(Attribute::new("job_condition_status", "valid"));
        if !resolution? {
            // TODO: do we want to return OK?
            // this means if a keeper accidentally executes a job whose condition is unmet
            // It still cost keeper TX fee
            // Shouldn't we return error so keeper will fail during simulation?
            return Ok(Response::new()
                .add_attribute("action", "execute_job")
                .add_attribute("condition", "false")
                .add_attribute("job_id", job.id));
        }

        submsgs.push(SubMsg {
            id: REPLY_ID_EXECUTE_JOB,
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: job.account.to_string(),
                msg: to_binary(&job_account::ExecuteMsg::Generic(GenericMsg {
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
    msgs.push(build_transfer_native_funds_msg(
        info.sender.to_string(),
        vec![Coin::new(job.reward.u128(), config.fee_denom)],
    ));

    if !is_legacy_account(legacy_account, job_account_addr.clone()) {
        // For job not using legacy account, job owner must already have account tracker instantiated
        let job_account_tracker = JOB_ACCOUNT_TRACKERS.load(deps.storage, &job.owner)?;
        // Free account
        msgs.push(build_free_account_msg(
            job_account_tracker.to_string(),
            job_account_addr.to_string(),
        ));
    }

    Ok(Response::new()
        .add_submessages(submsgs)
        .add_messages(msgs)
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
    config: Config,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    let job = JobQueue::get(&deps, data.id.into())?;
    let legacy_account = LEGACY_ACCOUNTS().may_load(deps.storage, job.owner.clone())?;
    let job_account_addr = job.account.clone();

    let account_amount = deps
        .querier
        .query::<BalanceResponse>(&QueryRequest::Bank(BankQuery::Balance {
            address: job_account_addr.clone().to_string(),
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
        return Err(ContractError::EvictionPeriodNotElapsed {});
    }

    let mut msgs = vec![];

    let job_status;

    if job.requeue_on_evict && account_amount >= a {
        // Job will stay active cause it has enough funds to pay for eviction fee and it's set to requeue on eviction
        msgs.push(
            // Job owner's warp account sends reward to evictor
            build_account_execute_generic_msgs(
                job_account_addr.to_string(),
                vec![build_transfer_native_funds_msg(
                    info.sender.to_string(),
                    vec![Coin::new(a.u128(), config.fee_denom)],
                )],
            ),
        );
        job_status = JobQueue::sync(&mut deps, env, job.clone())?.status;
    } else {
        // Job will be evicted
        job_status = JobQueue::finalize(&mut deps, env, job.id.into(), JobStatus::Evicted)?.status;

        // Controller sends eviction reward to evictor
        msgs.push(build_transfer_native_funds_msg(
            info.sender.to_string(),
            vec![Coin::new(a.u128(), config.fee_denom.clone())],
        ));

        // Controller sends execution reward minus eviction reward back to account
        msgs.push(build_transfer_native_funds_msg(
            info.sender.to_string(),
            vec![Coin::new((job.reward - a).u128(), config.fee_denom.clone())],
        ));

        if !is_legacy_account(legacy_account, job_account_addr.clone()) {
            // For job not using legacy account, job owner must already have account tracker instantiated
            let job_account_tracker = JOB_ACCOUNT_TRACKERS.load(deps.storage, &job.owner)?;
            // Free account
            msgs.push(build_free_account_msg(
                job_account_tracker.to_string(),
                job_account_addr.to_string(),
            ));
        }
    }

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "evict_job")
        .add_attribute("job_id", job.id)
        .add_attribute("job_status", serde_json_wasm::to_string(&job_status)?))
}

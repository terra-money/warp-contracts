use crate::contract::REPLY_ID_CREATE_JOB_ACCOUNT_AND_JOB;
use crate::state::{JobQueue, STATE};
use crate::util::msg::{
    build_account_execute_generic_msgs, build_account_execute_warp_msgs,
    build_free_funding_account_msg, build_take_funding_account_msg,
};
use crate::ContractError;
use controller::account::WarpMsgs;
use controller::job::{
    CreateJobMsg, DeleteJobMsg, EvictJobMsg, ExecuteJobMsg, Execution, Job, JobStatus, UpdateJobMsg,
};
use cosmwasm_std::{
    to_binary, Attribute, Coin, CosmosMsg, DepsMut, Env, MessageInfo, ReplyOn, Response, StdResult,
    SubMsg, Uint128, Uint64, WasmMsg,
};

use crate::util::{
    fee::deduct_from_native_funds,
    msg::{
        build_account_withdraw_assets_msg, build_free_job_account_msg,
        build_instantiate_warp_account_msg, build_take_job_account_msg, build_transfer_cw20_msg,
        build_transfer_cw721_msg, build_transfer_native_funds_msg,
    },
};

use account_tracker::{FundingAccount, FundingAccountResponse, JobAccountResponse};
use controller::{account::CwFund, Config};
use resolver::QueryHydrateMsgsMsg;

use super::fee::{compute_burn_fee, compute_creation_fee, compute_maintenance_fee};

const MAX_TEXT_LENGTH: usize = 280;

pub fn create_job(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: CreateJobMsg,
    config: Config,
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

    if data.duration_days > config.duration_days_limit {
        return Err(ContractError::DurationDaysLimit {});
    }

    let state = STATE.load(deps.storage)?;

    let job_owner = info.sender.clone();
    let account_tracker_address_ref = &config.account_tracker_address.to_string();

    let _validate_conditions_and_variables: Option<String> = deps.querier.query_wasm_smart(
        &config.resolver_address,
        &resolver::QueryMsg::QueryValidateJobCreation(resolver::QueryValidateJobCreationMsg {
            terminate_condition: data.terminate_condition.clone(),
            vars: data.vars.clone(),
            executions: data.executions.clone(),
        }),
    )?;

    let creation_fee = compute_creation_fee(state.q, &config);
    let maintenance_fee = compute_maintenance_fee(data.duration_days, &config);
    let burn_fee = compute_burn_fee(data.reward, &config);

    let total_fees = creation_fee + maintenance_fee + burn_fee;

    if data.funding_account.is_none() && data.recurring {
        return Err(ContractError::FundingAccountMissingForRecurringJob {});
    }

    if data.funding_account.is_none() {
        if data.operational_amount < total_fees + data.reward {
            return Err(ContractError::InsufficientOperationalFunds {});
        }

        let fee_denom_paid_amount = info
            .funds
            .iter()
            .find(|f| f.denom == config.fee_denom)
            .unwrap()
            .amount;

        if fee_denom_paid_amount < data.operational_amount {
            return Err(ContractError::InsufficientFundsToPayForRewardAndFee {});
        }
    }

    // ignore operational_amount when funding_account is provided
    let operational_amount = if data.funding_account.is_some() {
        Uint128::zero()
    } else {
        data.operational_amount
    };

    // Reward and fee will always be in native denom
    let native_funds_minus_operational_amount = deduct_from_native_funds(
        info.funds.clone(),
        config.fee_denom.clone(),
        operational_amount,
    );

    let mut submsgs = vec![];
    let mut msgs = vec![];
    let mut attrs = vec![];

    let mut job = JobQueue::add(
        deps.storage,
        Job {
            id: state.current_job_id,
            prev_id: None,
            owner: job_owner.clone(),
            // Account uses a placeholder value for now, will update it to job account address if job account exists or after created
            // Update will happen either in create_job (exists free job account) or reply (after creation), so it's atomic
            // And we guarantee we do not read this value before it's updated
            account: info.sender.clone(),
            last_update_time: Uint64::from(env.block.time.seconds()),
            name: data.name,
            status: JobStatus::Pending,
            terminate_condition: data.terminate_condition,
            recurring: data.recurring,
            vars: data.vars,
            executions: data.executions,
            reward: data.reward,
            description: data.description,
            labels: data.labels,
            assets_to_withdraw: data.assets_to_withdraw.unwrap_or(vec![]),
            duration_days: data.duration_days,
            created_at_time: Uint64::from(env.block.time.seconds()),
            // placeholder, will be updated later on
            funding_account: None,
        },
    )?;

    let job_account_resp: JobAccountResponse = deps.querier.query_wasm_smart(
        account_tracker_address_ref,
        &account_tracker::QueryMsg::QueryFirstFreeJobAccount(
            account_tracker::QueryFirstFreeJobAccountMsg {
                account_owner_addr: job_owner.to_string(),
            },
        ),
    )?;

    match job_account_resp.job_account {
        None => {
            // Create account then create job in reply
            submsgs.push(SubMsg {
                id: REPLY_ID_CREATE_JOB_ACCOUNT_AND_JOB,
                msg: build_instantiate_warp_account_msg(
                    job.id,
                    env.contract.address.to_string(),
                    config.warp_account_code_id.u64(),
                    info.sender.to_string(),
                    native_funds_minus_operational_amount,
                    data.cw_funds,
                    data.account_msgs,
                ),
                gas_limit: None,
                reply_on: ReplyOn::Always,
            });

            attrs.push(Attribute::new("action", "create_account_and_job"));
        }
        Some(available_account) => {
            let available_account_addr = &available_account.account_addr;
            // Update job.account from placeholder value to job account
            job.account = available_account_addr.clone();
            JobQueue::sync(deps.storage, env.clone(), job.clone())?;

            if !native_funds_minus_operational_amount.is_empty() {
                // Fund account in native coins
                msgs.push(build_transfer_native_funds_msg(
                    available_account_addr.to_string(),
                    native_funds_minus_operational_amount,
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
                msgs.push(build_account_execute_warp_msgs(
                    available_account_addr.to_string(),
                    account_msgs,
                ));
            }

            // Take account
            msgs.push(build_take_job_account_msg(
                config.account_tracker_address.to_string(),
                job_owner.to_string(),
                available_account_addr.to_string(),
                job.id,
            ));

            attrs.push(Attribute::new("action", "create_job"));
            attrs.push(Attribute::new("job_id", job.id));
            attrs.push(Attribute::new("job_owner", job.owner.clone()));
            attrs.push(Attribute::new("job_name", job.name.clone()));
            attrs.push(Attribute::new(
                "job_status",
                serde_json_wasm::to_string(&job.status)?,
            ));
            attrs.push(Attribute::new(
                "job_executions",
                serde_json_wasm::to_string(&job.executions)?,
            ));
            attrs.push(Attribute::new("job_reward", job.reward));
            attrs.push(Attribute::new("job_creation_fee", creation_fee.to_string()));
            attrs.push(Attribute::new(
                "job_maintenance_fee",
                maintenance_fee.to_string(),
            ));
            attrs.push(Attribute::new("job_burn_fee", burn_fee.to_string()));
            attrs.push(Attribute::new("job_total_fees", total_fees.to_string()));
            attrs.push(Attribute::new(
                "job_last_updated_time",
                job.last_update_time,
            ));
        }
    }

    let mut funding_account: Option<FundingAccount> = None;

    if let Some(funding_account_addr) = data.funding_account {
        // fetch funding account and check if it exists, throw otherwise
        let funding_account_resp: FundingAccountResponse = deps.querier.query_wasm_smart(
            account_tracker_address_ref,
            &account_tracker::QueryMsg::QueryFundingAccount(
                account_tracker::QueryFundingAccountMsg {
                    account_addr: funding_account_addr.to_string(),
                    account_owner_addr: info.sender.to_string(),
                },
            ),
        )?;

        funding_account = funding_account_resp.funding_account;
    }

    match funding_account {
        None => {
            // exit only applies for recurring jobs, otherwise funds are in controller
            if data.recurring {
                return Err(ContractError::FundingAccountMissingForRecurringJob {});
            }
        }
        Some(available_account) => {
            let available_account_addr = &available_account.account_addr;
            // Update funding_account from placeholder value to funding account
            job.funding_account = Some(available_account_addr.clone());
            JobQueue::sync(deps.storage, env.clone(), job.clone())?;

            // transfer reward + fees to controller from funding account
            msgs.push(build_account_execute_generic_msgs(
                job.funding_account.clone().unwrap().to_string(),
                vec![build_transfer_native_funds_msg(
                    env.contract.address.to_string(),
                    vec![Coin::new(
                        total_fees.u128() + data.reward.u128(),
                        config.fee_denom.clone(),
                    )],
                )],
            ));

            // Take account
            msgs.push(build_take_funding_account_msg(
                config.account_tracker_address.to_string(),
                job_owner.to_string(),
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
                "job_executions",
                serde_json_wasm::to_string(&job.executions)?,
            ));
            attrs.push(Attribute::new("job_reward", job.reward));
            attrs.push(Attribute::new("job_creation_fee", creation_fee.to_string()));
            attrs.push(Attribute::new(
                "job_maintenance_fee",
                maintenance_fee.to_string(),
            ));
            attrs.push(Attribute::new("job_burn_fee", burn_fee.to_string()));
            attrs.push(Attribute::new("job_total_fees", total_fees.to_string()));
            attrs.push(Attribute::new(
                "job_last_updated_time",
                job.last_update_time,
            ));
        }
    }

    // Job owner sends reward to controller when it calls create_job
    // Reward stays at controller, no need to send it elsewhere
    msgs.push(
        // Job owner sends fee to controller when it calls create_job
        // Controller sends fee to fee collector
        build_transfer_native_funds_msg(
            config.fee_collector.to_string(),
            vec![Coin::new(total_fees.u128(), config.fee_denom)],
        ),
    );

    Ok(Response::new()
        .add_submessages(submsgs)
        .add_messages(msgs)
        .add_attributes(attrs))
}

pub fn delete_job(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: DeleteJobMsg,
    config: Config,
) -> Result<Response, ContractError> {
    let job = JobQueue::get(deps.storage, data.id.into())?;
    let account_addr = job.account.clone();

    if job.status != JobStatus::Pending {
        return Err(ContractError::JobNotActive {});
    }

    if job.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let _new_job = JobQueue::finalize(deps.storage, env, job.id.into(), JobStatus::Cancelled)?;

    let fee = job.reward * Uint128::from(config.cancellation_fee_rate) / Uint128::new(100);

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
        vec![Coin::new(fee.u128(), config.fee_denom.clone())],
    ));

    // Free account
    msgs.push(build_free_job_account_msg(
        config.account_tracker_address.to_string(),
        job.owner.to_string(),
        account_addr.to_string(),
        job.id,
    ));

    if let Some(funding_account) = job.funding_account {
        msgs.push(build_free_funding_account_msg(
            config.account_tracker_address.to_string(),
            job.owner.to_string(),
            funding_account.to_string(),
            job.id,
        ));
    }

    // Job owner withdraw all assets that are listed from warp account to itself
    msgs.push(build_account_withdraw_assets_msg(
        account_addr.to_string(),
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
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: UpdateJobMsg,
) -> Result<Response, ContractError> {
    let job = JobQueue::get(deps.storage, data.id.into())?;

    if info.sender != job.owner {
        return Err(ContractError::Unauthorized {});
    }

    if data.name.is_some() && data.name.clone().unwrap().len() > MAX_TEXT_LENGTH {
        return Err(ContractError::NameTooLong {});
    }

    if data.name.is_some() && data.name.clone().unwrap().is_empty() {
        return Err(ContractError::NameTooShort {});
    }

    let job = JobQueue::update(deps.storage, env, data)?;

    Ok(Response::new()
        .add_attribute("action", "update_job")
        .add_attribute("job_id", job.id)
        .add_attribute("job_owner", job.owner)
        .add_attribute("job_name", job.name)
        .add_attribute("job_status", serde_json_wasm::to_string(&job.status)?)
        .add_attribute(
            "job_executions",
            serde_json_wasm::to_string(&job.executions)?,
        )
        .add_attribute("job_reward", job.reward)
        .add_attribute("job_last_updated_time", job.last_update_time))
}

pub fn execute_job(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: ExecuteJobMsg,
    config: Config,
) -> Result<Response, ContractError> {
    let job = JobQueue::get(deps.storage, data.id.into())?;
    let account_addr = job.account.clone();

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

    let mut attrs = vec![];
    let mut msgs = vec![];
    let mut submsgs = vec![];

    let mut execution_matched = false;

    for Execution { condition, msgs } in job.executions {
        let resolution: StdResult<bool> = deps.querier.query_wasm_smart(
            config.resolver_address.clone(),
            &resolver::QueryMsg::QueryResolveCondition(resolver::QueryResolveConditionMsg {
                condition,
                vars: vars.clone(),
                warp_account_addr: Some(job.account.to_string()),
            }),
        );

        match resolution {
            Ok(true) => {
                submsgs.push(SubMsg {
                    id: data.id.u64(),
                    msg: CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: job.account.to_string(),
                        msg: to_binary(&account::ExecuteMsg::WarpMsgs(WarpMsgs {
                            msgs: deps.querier.query_wasm_smart(
                                config.resolver_address,
                                &resolver::QueryMsg::QueryHydrateMsgs(QueryHydrateMsgsMsg {
                                    msgs,
                                    vars,
                                }),
                            )?,
                            job_id: Some(data.id),
                        }))?,
                        funds: vec![],
                    }),
                    gas_limit: None,
                    reply_on: ReplyOn::Always,
                });

                execution_matched = true;

                break;
            }
            Ok(false) => {
                // Continue to the next condition
                continue;
            }
            Err(e) => {
                attrs.push(Attribute::new("job_condition_status", "invalid"));
                attrs.push(Attribute::new("error", e.to_string()));
                JobQueue::finalize(deps.storage, env, job.id.into(), JobStatus::Failed)?;

                execution_matched = true;

                break;
            }
        }
    }

    if !execution_matched {
        return Ok(Response::new()
            .add_attribute("action", "execute_job")
            .add_attribute("executor", info.sender)
            .add_attribute("job_id", job.id)
            .add_attribute("job_condition", "inactive")
            .add_attributes(attrs));
    }

    // Controller sends reward to executor
    msgs.push(build_transfer_native_funds_msg(
        info.sender.to_string(),
        vec![Coin::new(job.reward.u128(), config.fee_denom)],
    ));

    // Free account
    msgs.push(build_free_job_account_msg(
        config.account_tracker_address.to_string(),
        job.owner.to_string(),
        account_addr.to_string(),
        job.id,
    ));

    if let Some(funding_account) = job.funding_account {
        msgs.push(build_free_funding_account_msg(
            config.account_tracker_address.to_string(),
            job.owner.to_string(),
            funding_account.to_string(),
            job.id,
        ));
    }

    Ok(Response::new()
        .add_messages(msgs)
        .add_submessages(submsgs)
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
    config: Config,
) -> Result<Response, ContractError> {
    let job = JobQueue::get(deps.storage, data.id.into())?;
    let account_addr = job.account.clone();

    if job.status != JobStatus::Pending {
        return Err(ContractError::Unauthorized {});
    }

    let eviction_fee = config.maintenance_fee_min;

    if (env.block.time.seconds() - job.created_at_time.u64()) < (job.duration_days.u64() * 86400) {
        return Err(ContractError::EvictionPeriodNotElapsed {});
    }

    let mut msgs = vec![];

    // Job will be evicted
    let job_status =
        JobQueue::finalize(deps.storage, env, job.id.into(), JobStatus::Evicted)?.status;

    // Controller sends eviction reward to evictor
    msgs.push(build_transfer_native_funds_msg(
        info.sender.to_string(),
        vec![Coin::new(eviction_fee.u128(), config.fee_denom.clone())],
    ));

    // Controller sends execution reward minus eviction reward back to owner
    msgs.push(build_transfer_native_funds_msg(
        job.owner.to_string(),
        vec![Coin::new(
            (job.reward - eviction_fee).u128(),
            config.fee_denom.clone(),
        )],
    ));

    // Free account
    msgs.push(build_free_job_account_msg(
        config.account_tracker_address.to_string(),
        job.owner.to_string(),
        account_addr.to_string(),
        job.id,
    ));

    if let Some(funding_account) = job.funding_account {
        msgs.push(build_free_funding_account_msg(
            config.account_tracker_address.to_string(),
            job.owner.to_string(),
            funding_account.to_string(),
            job.id,
        ));
    }

    // Job owner withdraw all assets that are listed from warp account to itself
    msgs.push(build_account_withdraw_assets_msg(
        account_addr.to_string(),
        job.assets_to_withdraw,
    ));

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "evict_job")
        .add_attribute("job_id", job.id)
        .add_attribute("job_status", serde_json_wasm::to_string(&job_status)?))
}

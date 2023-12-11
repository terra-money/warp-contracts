use cosmwasm_std::{
    Attribute, BalanceResponse, BankQuery, Coin, DepsMut, Env, QueryRequest, Reply, Response,
    StdError, StdResult, SubMsgResult, Uint64,
};

use crate::{
    error::map_contract_error,
    execute::fee::{compute_burn_fee, compute_creation_fee, compute_maintenance_fee},
    state::{JobQueue, CONFIG, STATE},
    util::msg::{
        build_account_execute_generic_msgs, build_account_withdraw_assets_msg,
        build_take_account_msg, build_take_funding_account_msg, build_transfer_native_funds_msg,
    },
    ContractError,
};
use controller::{
    account::AssetInfo,
    job::{Job, JobStatus},
    Config,
};

pub fn execute_job(
    deps: DepsMut,
    env: Env,
    msg: Reply,
    config: Config,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    let new_status = match msg.result {
        SubMsgResult::Ok(_) => JobStatus::Executed,
        SubMsgResult::Err(_) => JobStatus::Failed,
    };

    let reply: cosmwasm_std::SubMsgResponse = msg
        .result
        .clone()
        .into_result()
        .map_err(StdError::generic_err)?;

    let warp_msgs_event = reply
        .events
        .iter()
        .find(|event| {
            event
                .attributes
                .iter()
                .any(|attr| attr.key == "action" && attr.value == "warp_msgs")
        })
        .ok_or_else(|| StdError::generic_err("cannot find `warp_msgs` event"))?;

    let job_id_str = warp_msgs_event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "job_id")
        .ok_or_else(|| StdError::generic_err("cannot find `job_id` attribute"))?
        .value;

    let job_id = job_id_str.as_str().parse::<u64>()?;

    let finished_job = JobQueue::finalize(deps.storage, env.clone(), job_id, new_status)?;

    let res_attrs = match msg.result {
        SubMsgResult::Err(e) => vec![Attribute::new(
            "transaction_error",
            format!("{}. {}", &e, map_contract_error(&e)),
        )],
        _ => vec![],
    };

    let mut msgs = vec![];
    let mut new_job_attrs = vec![];
    let new_job_id = state.current_job_id;

    let creation_fee = compute_creation_fee(state.q, &config);
    let maintenance_fee = compute_maintenance_fee(finished_job.duration_days, &config);
    let burn_fee = compute_burn_fee(finished_job.reward, &config);

    let total_fees = creation_fee + maintenance_fee + burn_fee;

    let reward_plus_fee = finished_job.reward + total_fees;

    let account_addr = finished_job.account.clone();

    let mut recurring_job_created = false;

    if finished_job.recurring {
        let funding_account_addr = finished_job.funding_account.clone().unwrap();

        let operational_amount = deps
            .querier
            .query::<BalanceResponse>(&QueryRequest::Bank(BankQuery::Balance {
                address: funding_account_addr.to_string(),
                denom: config.fee_denom.clone(),
            }))?
            .amount
            .amount;

        if operational_amount < reward_plus_fee {
            new_job_attrs.push(Attribute::new("action", "recur_job"));
            new_job_attrs.push(Attribute::new("creation_status", "failed_insufficient_fee"));
        } else if !(finished_job.status == JobStatus::Executed
            || finished_job.status == JobStatus::Failed)
        {
            new_job_attrs.push(Attribute::new("action", "recur_job"));
            new_job_attrs.push(Attribute::new(
                "creation_status",
                "failed_invalid_job_status",
            ));
        } else {
            let new_vars: String = deps.querier.query_wasm_smart(
                config.resolver_address.clone(),
                &resolver::QueryMsg::QueryApplyVarFn(resolver::QueryApplyVarFnMsg {
                    vars: finished_job.vars,
                    status: finished_job.status.clone(),
                    warp_account_addr: Some(finished_job.account.to_string()),
                }),
            )?;

            let should_terminate_job: bool;
            match finished_job.terminate_condition.clone() {
                Some(terminate_condition) => {
                    let resolution: StdResult<bool> = deps.querier.query_wasm_smart(
                        config.resolver_address,
                        &resolver::QueryMsg::QueryResolveCondition(
                            resolver::QueryResolveConditionMsg {
                                condition: terminate_condition,
                                vars: new_vars.clone(),
                                warp_account_addr: Some(finished_job.account.to_string()),
                            },
                        ),
                    );
                    if let Err(e) = resolution {
                        should_terminate_job = true;
                        new_job_attrs.push(Attribute::new("action", "recur_job"));
                        new_job_attrs
                            .push(Attribute::new("job_terminate_condition_status", "invalid"));
                        new_job_attrs.push(Attribute::new(
                            "creation_status",
                            format!(
                                "terminated_due_to_terminate_condition_resolves_to_error. {}",
                                e
                            ),
                        ));
                    } else {
                        new_job_attrs
                            .push(Attribute::new("job_terminate_condition_status", "valid"));
                        if resolution? {
                            should_terminate_job = true;
                            new_job_attrs.push(Attribute::new("action", "recur_job"));
                            new_job_attrs.push(Attribute::new(
                                "creation_status",
                                "terminated_due_to_terminate_condition_resolves_to_true",
                            ));
                        } else {
                            should_terminate_job = false;
                        }
                    }
                }
                None => {
                    should_terminate_job = false;
                }
            }

            if !should_terminate_job {
                recurring_job_created = true;

                let operational_amount_minus_reward_and_fee =
                    operational_amount.checked_sub(finished_job.reward + total_fees)?;

                let new_job = JobQueue::add(
                    deps.storage,
                    Job {
                        id: new_job_id,
                        prev_id: Some(finished_job.id),
                        owner: finished_job.owner.clone(),
                        account: finished_job.account.clone(),
                        last_update_time: Uint64::from(env.block.time.seconds()),
                        name: finished_job.name.clone(),
                        description: finished_job.description,
                        labels: finished_job.labels,
                        status: JobStatus::Pending,
                        executions: finished_job.executions,
                        terminate_condition: finished_job.terminate_condition.clone(),
                        vars: new_vars,
                        recurring: finished_job.recurring,
                        reward: finished_job.reward,
                        operational_amount: operational_amount_minus_reward_and_fee,
                        assets_to_withdraw: finished_job.assets_to_withdraw.clone(),
                        duration_days: finished_job.duration_days,
                        created_at_time: Uint64::from(env.block.time.seconds()),
                        funding_account: finished_job.funding_account.clone(),
                    },
                )?;

                msgs.push(build_account_execute_generic_msgs(
                    funding_account_addr.to_string(),
                    vec![
                        // Job owner's funding account sends fee to fee collector
                        build_transfer_native_funds_msg(
                            config.fee_collector.to_string(),
                            vec![Coin::new(total_fees.u128(), config.fee_denom.clone())],
                        ),
                        // Job owner's funding account sends reward to controller
                        build_transfer_native_funds_msg(
                            env.contract.address.to_string(),
                            vec![Coin::new(new_job.reward.u128(), config.fee_denom.clone())],
                        ),
                    ],
                ));

                new_job_attrs.push(Attribute::new("action", "create_job"));
                new_job_attrs.push(Attribute::new("job_id", new_job.id));
                new_job_attrs.push(Attribute::new("job_owner", new_job.owner));
                new_job_attrs.push(Attribute::new("job_name", new_job.name));
                new_job_attrs.push(Attribute::new(
                    "job_status",
                    serde_json_wasm::to_string(&new_job.status)?,
                ));
                new_job_attrs.push(Attribute::new(
                    "job_executions",
                    serde_json_wasm::to_string(&new_job.executions)?,
                ));
                new_job_attrs.push(Attribute::new("job_reward", new_job.reward));
                new_job_attrs.push(Attribute::new("job_creation_fee", creation_fee.to_string()));
                new_job_attrs.push(Attribute::new(
                    "job_maintenance_fee",
                    maintenance_fee.to_string(),
                ));
                new_job_attrs.push(Attribute::new("job_burn_fee", burn_fee.to_string()));
                new_job_attrs.push(Attribute::new("job_total_fees", total_fees.to_string()));
                new_job_attrs.push(Attribute::new(
                    "job_last_updated_time",
                    new_job.last_update_time,
                ));
                new_job_attrs.push(Attribute::new("sub_action", "recur_job"));
            }
        }
    }

    if recurring_job_created {
        let funding_account_addr = finished_job.funding_account.clone().unwrap();

        // Take job account with the new job
        msgs.push(build_take_account_msg(
            config.account_tracker_address.to_string(),
            finished_job.owner.to_string(),
            account_addr.to_string(),
            new_job_id,
        ));

        // take funding account with new job
        msgs.push(build_take_funding_account_msg(
            config.account_tracker_address.to_string(),
            finished_job.owner.to_string(),
            funding_account_addr.to_string(),
            new_job_id,
        ));
    } else {
        // No new job created, account has been free in execute_job, no need to free here again
        // Job owner withdraw all assets that are listed from warp account to itself
        msgs.push(build_account_withdraw_assets_msg(
            account_addr.to_string(),
            finished_job.assets_to_withdraw,
        ));

        // withdraw all funds if funding acc exists
        if let Some(acc) = finished_job.funding_account {
            msgs.push(build_account_withdraw_assets_msg(
                acc.to_string(),
                vec![AssetInfo::Native(config.fee_denom)],
            ));
        }
    }

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "execute_job_reply")
        .add_attribute("job_id", finished_job.id)
        .add_attributes(res_attrs)
        .add_attributes(new_job_attrs))
}

pub fn instantiate_sub_contracts(
    deps: DepsMut,
    _env: Env,
    msg: Reply,
    mut config: Config,
) -> Result<Response, ContractError> {
    let reply: cosmwasm_std::SubMsgResponse =
        msg.result.into_result().map_err(StdError::generic_err)?;

    let account_tracker_instantiate_event = reply
        .events
        .iter()
        .find(|event| {
            event
                .attributes
                .iter()
                .any(|attr| attr.key == "action" && attr.value == "instantiate")
        })
        .ok_or_else(|| StdError::generic_err("cannot find `instantiate` event"))?;

    let account_tracker_addr = deps.api.addr_validate(
        &account_tracker_instantiate_event
            .attributes
            .iter()
            .cloned()
            .find(|attr| attr.key == "account_tracker")
            .ok_or_else(|| StdError::generic_err("cannot find `account_tracker` attribute"))?
            .value,
    )?;

    config.account_tracker_address = account_tracker_addr;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "instantiate_sub_contracts_reply"))
}

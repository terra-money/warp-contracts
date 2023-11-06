use cosmwasm_std::{
    Attribute, BalanceResponse, BankQuery, Coin, DepsMut, Env, QueryRequest, Reply, Response,
    StdError, StdResult, SubMsgResult, Uint128, Uint64,
};

use crate::{
    error::map_contract_error,
    state::{JobQueue, LEGACY_ACCOUNTS, STATE},
    util::{
        legacy_account::is_legacy_account,
        msg::{
            build_account_execute_generic_msgs, build_account_withdraw_assets_msg,
            build_taken_account_msg, build_transfer_native_funds_msg,
        },
    },
    ContractError,
};
use controller::{
    job::{Job, JobStatus},
    Config,
};

pub fn execute_job(
    mut deps: DepsMut,
    env: Env,
    msg: Reply,
    config: Config,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    let new_status = match msg.result {
        SubMsgResult::Ok(_) => JobStatus::Executed,
        SubMsgResult::Err(_) => JobStatus::Failed,
    };

    let reply = msg
        .result
        .clone()
        .into_result()
        .map_err(StdError::generic_err)?;

    // TODO: how to get this event
    let event = reply
        .events
        .iter()
        .find(|event| {
            event
                .attributes
                .iter()
                .any(|attr| attr.key == "action" && attr.value == "execute_job")
        })
        .ok_or_else(|| StdError::generic_err("cannot find `execute_job` event"))?;

    let job_id_str = event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "job_id")
        .ok_or_else(|| StdError::generic_err("cannot find `job_id` attribute"))?
        .value;

    let job_id = job_id_str.as_str().parse::<u64>()?;

    let finished_job = JobQueue::finalize(&mut deps, env.clone(), job_id, new_status)?;

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

    let fee =
        finished_job.reward * Uint128::from(config.creation_fee_percentage) / Uint128::new(100);
    let fee_plus_reward = fee + finished_job.reward;

    let legacy_account = LEGACY_ACCOUNTS().may_load(deps.storage, finished_job.owner.clone())?;
    let job_account_addr = finished_job.account.clone();

    let job_account_amount = deps
        .querier
        .query::<BalanceResponse>(&QueryRequest::Bank(BankQuery::Balance {
            address: job_account_addr.to_string(),
            denom: config.fee_denom.clone(),
        }))?
        .amount
        .amount;

    let mut recurring_job_created = false;

    if finished_job.recurring {
        if job_account_amount < fee_plus_reward {
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
                    warp_account_addr: Some(finished_job.account.clone().to_string()),
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
                let new_job = JobQueue::add(
                    &mut deps,
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
                        requeue_on_evict: finished_job.requeue_on_evict,
                        recurring: finished_job.recurring,
                        reward: finished_job.reward,
                        assets_to_withdraw: finished_job.assets_to_withdraw.clone(),
                    },
                )?;

                msgs.push(build_account_execute_generic_msgs(
                    job_account_addr.clone().to_string(),
                    vec![
                        // Job owner's warp account sends fee to fee collector
                        build_transfer_native_funds_msg(
                            config.fee_collector.to_string(),
                            vec![Coin::new(fee.u128(), config.fee_denom.clone())],
                        ),
                        // Job owner's warp account sends reward to controller
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
                new_job_attrs.push(Attribute::new("job_creation_fee", fee));
                new_job_attrs.push(Attribute::new(
                    "job_last_updated_time",
                    new_job.last_update_time,
                ));
                new_job_attrs.push(Attribute::new("sub_action", "recur_job"));
            }
        }
    }

    if recurring_job_created {
        if !is_legacy_account(legacy_account, job_account_addr.clone()) {
            // Take job account with the new job
            msgs.push(build_taken_account_msg(
                config.job_account_tracker_address.to_string(),
                finished_job.owner.to_string(),
                job_account_addr.to_string(),
                new_job_id,
            ));
        }
    } else {
        // No new job created, account has been free in execute_job, no need to free here again
        // Job owner withdraw all assets that are listed from warp account to itself
        msgs.push(build_account_withdraw_assets_msg(
            job_account_addr.clone().to_string(),
            finished_job.assets_to_withdraw,
        ));
    }

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "execute_job_reply")
        .add_attribute("job_id", finished_job.id)
        .add_attributes(res_attrs)
        .add_attributes(new_job_attrs))
}

use cosmwasm_std::{
    Attribute, BalanceResponse, BankQuery, Coin, DepsMut, Env, QueryRequest, Reply, Response,
    StdResult, SubMsgResult, Uint128, Uint64,
};

use crate::{
    error::map_contract_error,
    state::{JobQueue, ACCOUNTS, STATE},
    util::{
        msg::{
            build_account_execute_generic_msgs, build_account_withdraw_assets_msg,
            build_free_sub_account_msg, build_transfer_native_funds_msg,
        },
        sub_account::is_sub_account,
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

    let finished_job = JobQueue::finalize(&mut deps, env.clone(), msg.id, new_status)?;

    let res_attrs = match msg.result {
        SubMsgResult::Err(e) => vec![Attribute::new(
            "transaction_error",
            format!("{}. {}", &e, map_contract_error(&e)),
        )],
        _ => vec![],
    };

    let mut msgs = vec![];
    let mut new_job_attrs = vec![];

    let fee =
        finished_job.reward * Uint128::from(config.creation_fee_percentage) / Uint128::new(100);
    let fee_plus_reward = fee + finished_job.reward;

    let main_account_addr = ACCOUNTS()
        .load(deps.storage, finished_job.owner.clone())?
        .account;
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
                        id: state.current_job_id,
                        prev_id: Some(finished_job.id),
                        owner: finished_job.owner.clone(),
                        account: finished_job.account.clone(),
                        last_update_time: Uint64::from(env.block.time.seconds()),
                        name: finished_job.name.clone(),
                        description: finished_job.description,
                        labels: finished_job.labels,
                        status: JobStatus::Pending,
                        condition: finished_job.condition.clone(),
                        terminate_condition: finished_job.terminate_condition.clone(),
                        vars: new_vars,
                        requeue_on_evict: finished_job.requeue_on_evict,
                        recurring: finished_job.recurring,
                        msgs: finished_job.msgs.clone(),
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
                    "job_condition",
                    serde_json_wasm::to_string(&new_job.condition)?,
                ));
                new_job_attrs.push(Attribute::new(
                    "job_msgs",
                    serde_json_wasm::to_string(&new_job.msgs)?,
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

    if !recurring_job_created {
        // Job owner withdraw all assets that are listed from warp account to itself
        msgs.push(build_account_withdraw_assets_msg(
            job_account_addr.clone().to_string(),
            finished_job.assets_to_withdraw,
        ));
        if is_sub_account(&main_account_addr, &job_account_addr) {
            // Free sub account
            msgs.push(build_free_sub_account_msg(
                main_account_addr.to_string(),
                job_account_addr.clone().to_string(),
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

use account::{AddInUseSubAccountMsg, FreeInUseSubAccountMsg, GenericMsg, WithdrawAssetsMsg};
use controller::job::{Job, JobStatus};
use cosmwasm_std::{
    to_binary, Attribute, BalanceResponse, BankMsg, BankQuery, Coin, CosmosMsg, DepsMut, Env,
    QueryRequest, Reply, Response, StdError, StdResult, SubMsgResult, Uint128, Uint64, WasmMsg,
};

use crate::{
    error::map_contract_error,
    state::{ACCOUNTS, CONFIG, FINISHED_JOBS, PENDING_JOBS, STATE},
    ContractError,
};

pub fn execute_job(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;

    let new_status = match msg.result {
        SubMsgResult::Ok(_) => JobStatus::Executed,
        SubMsgResult::Err(_) => JobStatus::Failed,
    };

    let reply = msg
        .result
        .clone()
        .into_result()
        .map_err(|e| StdError::generic_err(e))?;
    let event = reply
        .events
        .iter()
        .find(|event| {
            event
                .attributes
                .iter()
                .any(|attr| attr.key == "action" && attr.value == "generic")
        })
        .ok_or_else(|| StdError::generic_err("cannot find `generic` event"))?;
    let job_id_str = event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "job_id")
        .ok_or_else(|| StdError::generic_err("cannot find `job_id` attribute"))?
        .value;
    if job_id_str == "0" {
        return Err(ContractError::JobExecutionReplyHasInvalidJobId {});
    }
    let job_id = u64::from_str_radix(job_id_str.as_str(), 10)?;
    let job = PENDING_JOBS().load(deps.storage, job_id)?;

    PENDING_JOBS().remove(deps.storage, job_id)?;

    state.q = state.q.checked_sub(Uint64::new(1))?;

    let finished_job = FINISHED_JOBS().update(deps.storage, job_id, |j| match j {
        None => Ok(Job {
            id: job.id,
            prev_id: job.prev_id,
            owner: job.owner,
            account: job.account,
            last_update_time: job.last_update_time,
            name: job.name,
            description: job.description,
            labels: job.labels,
            status: new_status,
            condition: job.condition,
            terminate_condition: job.terminate_condition,
            msgs: job.msgs,
            vars: job.vars,
            recurring: job.recurring,
            requeue_on_evict: job.requeue_on_evict,
            reward: job.reward,
            assets_to_withdraw: job.assets_to_withdraw,
        }),
        Some(_) => Err(ContractError::JobAlreadyFinished {}),
    })?;

    let res_attrs = match msg.result {
        SubMsgResult::Err(e) => vec![Attribute::new(
            "transaction_error",
            format!("{}. {}", &e, map_contract_error(&e)),
        )],
        _ => vec![],
    };

    let mut msgs_vec = vec![];
    let mut new_job_attrs = vec![];

    let account;
    match finished_job.account.clone() {
        Some(a) => {
            account = a;
            msgs_vec.push(
                // Free account from in use account list
                // If account is default account, it will be ignored by the account contract
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: account.to_string(),
                    msg: to_binary(&account::ExecuteMsg::FreeInUseSubAccount(
                        FreeInUseSubAccountMsg {
                            sub_account: account.to_string(),
                        },
                    ))?,
                    funds: vec![],
                }),
            )
        }
        None => {
            account = ACCOUNTS()
                .load(deps.storage, finished_job.owner.clone())?
                .account;
        }
    }
    let config = CONFIG.load(deps.storage)?;

    //assume reward.amount == warp token allowance
    let fee =
        finished_job.reward * Uint128::from(config.creation_fee_percentage) / Uint128::new(100);

    let account_amount = deps
        .querier
        .query::<BalanceResponse>(&QueryRequest::Bank(BankQuery::Balance {
            address: account.to_string(),
            denom: config.fee_denom.clone(),
        }))?
        .amount
        .amount;

    // Only not withdraw asset when job is recurring and should not terminate
    let mut should_withdraw_asset = true;

    if finished_job.recurring {
        if account_amount < fee + finished_job.reward {
            new_job_attrs.push(Attribute::new("action", "recur_job"));
            new_job_attrs.push(Attribute::new("creation_status", "failed_insufficient_fee"))
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
                }),
            )?; //todo: TEST THIS

            let should_terminate_job: bool;
            match finished_job.terminate_condition.clone() {
                Some(terminate_condition) => {
                    let resolution: StdResult<bool> = deps.querier.query_wasm_smart(
                        config.resolver_address,
                        &resolver::QueryMsg::QueryResolveCondition(
                            resolver::QueryResolveConditionMsg {
                                condition: terminate_condition,
                                vars: new_vars.clone(),
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
                                e.to_string()
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
                should_withdraw_asset = false;
                let new_job = PENDING_JOBS().update(
                    deps.storage,
                    state.current_job_id.u64(),
                    |s| match s {
                        None => Ok(Job {
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
                        }),
                        Some(_) => Err(ContractError::JobAlreadyExists {}),
                    },
                )?;

                state.current_job_id = state.current_job_id.checked_add(Uint64::new(1))?;
                state.q = state.q.checked_add(Uint64::new(1))?;

                msgs_vec.push(
                    //send fee to fee collector
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: account.to_string(),
                        msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {
                            job_id: Some(new_job.id),
                            msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                                to_address: config.fee_collector.to_string(),
                                amount: vec![Coin::new((fee).u128(), config.fee_denom.clone())],
                            })],
                        }))?,
                        funds: vec![],
                    }),
                );

                msgs_vec.push(
                    //send reward to controller
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: account.to_string(),
                        msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {
                            job_id: Some(new_job.id),
                            msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                                to_address: env.contract.address.to_string(),
                                amount: vec![Coin::new(
                                    (new_job.reward).u128(),
                                    config.fee_denom.clone(),
                                )],
                            })],
                        }))?,
                        funds: vec![],
                    }),
                );

                if new_job.account.is_some() {
                    msgs_vec.push(
                        // Add account to in use account list
                        // If account is default account, it will be ignored by the account contract
                        CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: account.to_string(),
                            msg: to_binary(&account::ExecuteMsg::AddInUseSubAccount(
                                AddInUseSubAccountMsg {
                                    sub_account: account.to_string(),
                                    job_id: new_job.id,
                                },
                            ))?,
                            funds: vec![],
                        }),
                    )
                }

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

    if should_withdraw_asset {
        msgs_vec.push(
            //withdraw all assets that are listed
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: account.to_string(),
                msg: to_binary(&account::ExecuteMsg::WithdrawAssets(WithdrawAssetsMsg {
                    asset_infos: finished_job.assets_to_withdraw,
                }))?,
                funds: vec![],
            }),
        );
    }

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_messages(msgs_vec)
        .add_attribute("action", "execute_reply")
        .add_attribute("job_id", job.id)
        .add_attributes(res_attrs)
        .add_attributes(new_job_attrs))
}

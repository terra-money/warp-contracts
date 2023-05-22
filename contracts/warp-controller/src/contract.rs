use crate::execute::{account, controller, job};
use crate::query::condition;

use crate::execute::template::{delete_template, edit_template, submit_template};
use crate::query::template::{query_template, query_templates};
use crate::state::{ACCOUNTS, CONFIG, FINISHED_JOBS, PENDING_JOBS};
use crate::util::variable::apply_var_fn;
use crate::{query, state::STATE, ContractError};
use cosmwasm_std::{
    entry_point, to_binary, Attribute, BalanceResponse, BankMsg, BankQuery, Binary, Coin,
    CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest, Reply, Response, StdError, StdResult,
    SubMsgResult, Uint128, Uint64, WasmMsg,
};
use warp_protocol::controller::account::Account;
use warp_protocol::controller::job::{Job, JobStatus};
use warp_protocol::controller::{Config, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, State};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        current_job_id: Uint64::one(),
        current_template_id: Uint64::zero(),
        q: Uint64::zero(),
    };

    let config = Config {
        owner: deps
            .api
            .addr_validate(&msg.owner.unwrap_or_else(|| info.sender.to_string()))?,
        fee_collector: deps
            .api
            .addr_validate(&msg.fee_collector.unwrap_or_else(|| info.sender.to_string()))?,
        warp_account_code_id: msg.warp_account_code_id,
        minimum_reward: msg.minimum_reward,
        creation_fee_percentage: msg.creation_fee,
        cancellation_fee_percentage: msg.cancellation_fee,
        template_fee: msg.template_fee,
        t_max: msg.t_max,
        t_min: msg.t_min,
        a_max: msg.a_max,
        a_min: msg.a_min,
        q_max: msg.q_max,
    };

    if config.a_max < config.a_min {
        return Err(ContractError::MaxFeeUnderMinFee {});
    }

    if config.t_max < config.t_min {
        return Err(ContractError::MaxTimeUnderMinTime {});
    }

    if config.minimum_reward < config.a_min {
        return Err(ContractError::RewardSmallerThanFee {});
    }

    if config.creation_fee_percentage.u64() > 100 {
        return Err(ContractError::CreationFeeTooHigh {});
    }

    if config.cancellation_fee_percentage.u64() > 100 {
        return Err(ContractError::CancellationFeeTooHigh {});
    }

    STATE.save(deps.storage, &state)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateJob(data) => job::create_job(deps, env, info, data),
        ExecuteMsg::DeleteJob(data) => job::delete_job(deps, env, info, data),
        ExecuteMsg::UpdateJob(data) => job::update_job(deps, env, info, data),
        ExecuteMsg::ExecuteJob(data) => job::execute_job(deps, env, info, data),
        ExecuteMsg::EvictJob(data) => job::evict_job(deps, env, info, data),

        ExecuteMsg::CreateAccount(_) => account::create_account(deps, env, info),

        ExecuteMsg::UpdateConfig(data) => controller::update_config(deps, env, info, data),

        ExecuteMsg::SubmitTemplate(data) => submit_template(deps, env, info, data),
        ExecuteMsg::EditTemplate(data) => edit_template(deps, env, info, data),
        ExecuteMsg::DeleteTemplate(data) => delete_template(deps, env, info, data),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryResolveJobCondition(data) => {
            to_binary(&condition::query_condition_active(deps, env, data)?)
        }
        QueryMsg::QueryJob(data) => to_binary(&query::job::query_job(deps, env, data)?),
        QueryMsg::QueryJobs(data) => to_binary(&query::job::query_jobs(deps, env, data)?),
        QueryMsg::QueryResolveCondition(data) => {
            to_binary(&condition::query_resolve_condition(deps, env, data)?)
        }

        QueryMsg::SimulateQuery(data) => {
            to_binary(&query::controller::query_simulate_query(deps, env, data)?)
        }

        QueryMsg::QueryAccount(data) => to_binary(&query::account::query_account(deps, env, data)?),
        QueryMsg::QueryAccounts(data) => {
            to_binary(&query::account::query_accounts(deps, env, data)?)
        }

        QueryMsg::QueryConfig(data) => {
            to_binary(&query::controller::query_config(deps, env, data)?)
        }

        QueryMsg::QueryTemplate(data) => to_binary(&query_template(deps, env, data)?),
        QueryMsg::QueryTemplates(data) => to_binary(&query_templates(deps, env, data)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        //account creation
        0 => {
            let reply = msg.result.into_result().map_err(StdError::generic_err)?;

            let event = reply
                .events
                .iter()
                .find(|event| {
                    event
                        .attributes
                        .iter()
                        .any(|attr| attr.key == "action" && attr.value == "instantiate")
                })
                .ok_or_else(|| StdError::generic_err("cannot find `instantiate` event"))?;

            let owner = event
                .attributes
                .iter()
                .cloned()
                .find(|attr| attr.key == "owner")
                .ok_or_else(|| StdError::generic_err("cannot find `owner` attribute"))?
                .value;

            let address = event
                .attributes
                .iter()
                .cloned()
                .find(|attr| attr.key == "contract_addr")
                .ok_or_else(|| StdError::generic_err("cannot find `contract_addr` attribute"))?
                .value;

            if ACCOUNTS().has(deps.storage, deps.api.addr_validate(&owner)?) {
                return Err(ContractError::AccountAlreadyExists {});
            }

            ACCOUNTS().save(
                deps.storage,
                deps.api.addr_validate(&owner)?,
                &Account {
                    owner: deps.api.addr_validate(&owner.clone())?,
                    account: deps.api.addr_validate(&address)?,
                },
            )?;
            Ok(Response::new()
                .add_attribute("action", "save_account")
                .add_attribute("owner", owner)
                .add_attribute("account_address", address))
        }
        //job execution
        _ => {
            let mut state = STATE.load(deps.storage)?;

            let new_status = match msg.result {
                SubMsgResult::Ok(_) => JobStatus::Executed,
                SubMsgResult::Err(_) => JobStatus::Failed,
            };

            let job = PENDING_JOBS().load(deps.storage, msg.id)?;
            PENDING_JOBS().remove(deps.storage, msg.id)?;

            state.q = state.q.checked_sub(Uint64::new(1))?;

            let new_job = FINISHED_JOBS().update(deps.storage, msg.id, |j| match j {
                None => Ok(Job {
                    id: job.id,
                    owner: job.owner,
                    last_update_time: job.last_update_time,
                    name: job.name,
                    description: job.description,
                    labels: job.labels,
                    status: new_status,
                    condition: job.condition,
                    msgs: job.msgs,
                    vars: job.vars,
                    recurring: job.recurring,
                    requeue_on_evict: job.requeue_on_evict,
                    reward: job.reward,
                }),
                Some(_) => Err(ContractError::JobAlreadyFinished {}),
            })?;

            let res_attrs = match msg.result {
                SubMsgResult::Err(e) => vec![Attribute::new("transaction_error", e)],
                _ => vec![],
            };

            let mut msgs = vec![];
            let mut new_job_attrs = vec![];

            let account = ACCOUNTS().load(deps.storage, new_job.owner.clone())?;
            let config = CONFIG.load(deps.storage)?;

            //assume reward.amount == warp token allowance
            let fee =
                new_job.reward * Uint128::from(config.creation_fee_percentage) / Uint128::new(100);

            let account_amount = deps
                .querier
                .query::<BalanceResponse>(&QueryRequest::Bank(BankQuery::Balance {
                    address: account.account.to_string(),
                    denom: "uluna".to_string(),
                }))?
                .amount
                .amount;

            if new_job.recurring {
                if account_amount < fee + new_job.reward {
                    new_job_attrs.push(Attribute::new("action", "recur_job"));
                    new_job_attrs
                        .push(Attribute::new("creation_status", "failed_insufficient_fee"));
                } else if !(new_job.status == JobStatus::Executed
                    || new_job.status == JobStatus::Failed)
                {
                    new_job_attrs.push(Attribute::new("action", "recur_job"));
                    new_job_attrs.push(Attribute::new(
                        "creation_status",
                        "failed_invalid_job_status",
                    ));
                } else {
                    let new_vars =
                        apply_var_fn(deps.as_ref(), env.clone(), new_job.vars, new_job.status)?;
                    let job = PENDING_JOBS().update(
                        deps.storage,
                        state.current_job_id.u64(),
                        |s| match s {
                            None => Ok(Job {
                                id: state.current_job_id,
                                owner: new_job.owner,
                                last_update_time: Uint64::from(env.block.time.seconds()),
                                name: new_job.name,
                                description: new_job.description,
                                labels: new_job.labels,
                                status: JobStatus::Pending,
                                condition: new_job.condition.clone(),
                                vars: new_vars,
                                requeue_on_evict: new_job.requeue_on_evict,
                                recurring: new_job.recurring,
                                msgs: new_job.msgs,
                                reward: new_job.reward,
                            }),
                            Some(_) => Err(ContractError::JobAlreadyExists {}),
                        },
                    )?;

                    state.current_job_id = state.current_job_id.checked_add(Uint64::new(1))?;
                    state.q = state.q.checked_add(Uint64::new(1))?;

                    msgs.push(
                        //send reward to controller
                        WasmMsg::Execute {
                            contract_addr: account.account.to_string(),
                            msg: to_binary(&warp_protocol::account::ExecuteMsg {
                                msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                                    to_address: config.fee_collector.to_string(),
                                    amount: vec![Coin::new((fee).u128(), "uluna")],
                                })],
                            })?,
                            funds: vec![],
                        },
                    );

                    msgs.push(
                        //send reward to controller
                        WasmMsg::Execute {
                            contract_addr: account.account.to_string(),
                            msg: to_binary(&warp_protocol::account::ExecuteMsg {
                                msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                                    to_address: env.contract.address.to_string(),
                                    amount: vec![Coin::new((new_job.reward).u128(), "uluna")],
                                })],
                            })?,
                            funds: vec![],
                        },
                    );

                    new_job_attrs.push(Attribute::new("action", "recur_job"));
                    new_job_attrs.push(Attribute::new("creation_status", "created"));
                    new_job_attrs.push(Attribute::new("job_id", job.id));
                }
            }

            STATE.save(deps.storage, &state)?;

            Ok(Response::new()
                .add_attribute("action", "execute_reply")
                .add_attribute("job_id", job.id)
                .add_attribute("job_status", serde_json_wasm::to_string(&job.status)?)
                .add_attributes(res_attrs)
                .add_attributes(new_job_attrs)
                .add_messages(msgs))
        }
    }
}

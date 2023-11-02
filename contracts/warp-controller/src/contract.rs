use crate::error::map_contract_error;
use crate::state::{JobQueue, ACCOUNTS, CONFIG};
use crate::{execute, query, state::STATE, ContractError};
use account::{GenericMsg, WithdrawAssetsMsg};
use controller::account::{Account, Fund, FundTransferMsgs, TransferFromMsg, TransferNftMsg};
use controller::job::{Job, JobStatus};
use cosmwasm_schema::cw_serde;

use controller::{Config, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, State};
use cosmwasm_std::{
    entry_point, to_binary, Addr, Attribute, BalanceResponse, BankMsg, BankQuery, Binary, Coin,
    CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest, Reply, Response, StdError, StdResult,
    SubMsgResult, Uint128, Uint64, WasmMsg,
};
use cw_storage_plus::Item;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        current_job_id: Uint64::one(),
        q: Uint64::zero(),
    };

    let config = Config {
        owner: deps
            .api
            .addr_validate(&msg.owner.unwrap_or_else(|| info.sender.to_string()))?,
        fee_denom: msg.fee_denom,
        fee_collector: deps
            .api
            .addr_validate(&msg.fee_collector.unwrap_or_else(|| info.sender.to_string()))?,
        warp_account_code_id: msg.warp_account_code_id,
        minimum_reward: msg.minimum_reward,
        creation_fee_percentage: msg.creation_fee,
        cancellation_fee_percentage: msg.cancellation_fee,
        resolver_address: deps.api.addr_validate(&msg.resolver_address)?,
        t_max: msg.t_max,
        t_min: msg.t_min,
        a_max: msg.a_max,
        a_min: msg.a_min,
        q_max: msg.q_max,
        creation_fee_min: msg.creation_fee_min,
        creation_fee_max: msg.creation_fee_max,
        burn_fee_min: msg.burn_fee_min,
        maintenance_fee_min: msg.maintenance_fee_min,
        maintenance_fee_max: msg.maintenance_fee_max,
        duration_days_left: msg.duration_days_left,
        duration_days_right: msg.duration_days_right,
        queue_size_left: msg.queue_size_left,
        queue_size_right: msg.queue_size_right,
        burn_fee_rate: msg.burn_fee_rate,
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
        ExecuteMsg::CreateJob(data) => execute::job::create_job(deps, env, info, data),
        ExecuteMsg::DeleteJob(data) => execute::job::delete_job(deps, env, info, data),
        ExecuteMsg::UpdateJob(data) => execute::job::update_job(deps, env, info, data),
        ExecuteMsg::ExecuteJob(data) => execute::job::execute_job(deps, env, info, data),
        ExecuteMsg::EvictJob(data) => execute::job::evict_job(deps, env, info, data),

        ExecuteMsg::CreateAccount(data) => execute::account::create_account(deps, env, info, data),

        ExecuteMsg::UpdateConfig(data) => execute::controller::update_config(deps, env, info, data),

        ExecuteMsg::MigrateAccounts(data) => {
            execute::controller::migrate_accounts(deps, env, info, data)
        }
        ExecuteMsg::MigratePendingJobs(data) => {
            execute::controller::migrate_pending_jobs(deps, env, info, data)
        }
        ExecuteMsg::MigrateFinishedJobs(data) => {
            execute::controller::migrate_finished_jobs(deps, env, info, data)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryJob(data) => to_binary(&query::job::query_job(deps, env, data)?),
        QueryMsg::QueryJobs(data) => to_binary(&query::job::query_jobs(deps, env, data)?),

        QueryMsg::QueryAccount(data) => to_binary(&query::account::query_account(deps, env, data)?),
        QueryMsg::QueryAccounts(data) => {
            to_binary(&query::account::query_accounts(deps, env, data)?)
        }

        QueryMsg::QueryConfig(data) => {
            to_binary(&query::controller::query_config(deps, env, data)?)
        }
        QueryMsg::QueryState(data) => to_binary(&query::controller::query_state(deps, env, data)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    //CONFIG
    #[cw_serde]
    pub struct V1Config {
        pub owner: Addr,
        pub fee_denom: String,
        pub fee_collector: Addr,
        pub warp_account_code_id: Uint64,
        pub minimum_reward: Uint128,
        pub creation_fee_percentage: Uint64,
        pub cancellation_fee_percentage: Uint64,
        pub resolver_address: Addr,
        // maximum time for evictions
        pub t_max: Uint64,
        // minimum time for evictions
        pub t_min: Uint64,
        // maximum fee for evictions
        pub a_max: Uint128,
        // minimum fee for evictions
        pub a_min: Uint128,
        // maximum length of queue modifier for evictions
        pub q_max: Uint64,
    }

    const V1CONFIG: Item<V1Config> = Item::new("config");

    let v1_config = V1CONFIG.load(deps.storage)?;

    CONFIG.save(
        deps.storage,
        &Config {
            owner: v1_config.owner,
            fee_denom: v1_config.fee_denom,
            fee_collector: v1_config.fee_collector,
            warp_account_code_id: v1_config.warp_account_code_id,
            minimum_reward: v1_config.minimum_reward,
            creation_fee_percentage: v1_config.creation_fee_percentage,
            cancellation_fee_percentage: v1_config.cancellation_fee_percentage,
            resolver_address: v1_config.resolver_address,
            t_max: v1_config.t_max,
            t_min: v1_config.t_min,
            a_max: v1_config.a_max,
            a_min: v1_config.a_min,
            q_max: v1_config.q_max,
            creation_fee_min: msg.creation_fee_min,
            creation_fee_max: msg.creation_fee_max,
            burn_fee_min: msg.burn_fee_min,
            maintenance_fee_min: msg.maintenance_fee_min,
            maintenance_fee_max: msg.maintenance_fee_max,
            duration_days_left: msg.duration_days_left,
            duration_days_right: msg.duration_days_right,
            queue_size_left: msg.queue_size_left,
            queue_size_right: msg.queue_size_right,
            burn_fee_rate: msg.burn_fee_rate,
        },
    )?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(mut deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        // Account creation
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

            let funds: Vec<Coin> = serde_json_wasm::from_str(
                &event
                    .attributes
                    .iter()
                    .cloned()
                    .find(|attr| attr.key == "funds")
                    .ok_or_else(|| StdError::generic_err("cannot find `funds` attribute"))?
                    .value,
            )?;

            let cw_funds: Option<Vec<Fund>> = serde_json_wasm::from_str(
                &event
                    .attributes
                    .iter()
                    .cloned()
                    .find(|attr| attr.key == "cw_funds")
                    .ok_or_else(|| StdError::generic_err("cannot find `cw_funds` attribute"))?
                    .value,
            )?;

            let account_msgs: Option<Vec<CosmosMsg>> = serde_json_wasm::from_str(
                &event
                    .attributes
                    .iter()
                    .cloned()
                    .find(|attr| attr.key == "account_msgs")
                    .ok_or_else(|| StdError::generic_err("cannot find `account_msgs` attribute"))?
                    .value,
            )?;

            let cw_funds_vec = match cw_funds {
                None => {
                    vec![]
                }
                Some(funds) => funds,
            };

            let mut msgs_vec: Vec<CosmosMsg> = vec![];

            for cw_fund in &cw_funds_vec {
                msgs_vec.push(CosmosMsg::Wasm(match cw_fund {
                    Fund::Cw20(cw20_fund) => WasmMsg::Execute {
                        contract_addr: deps
                            .api
                            .addr_validate(&cw20_fund.contract_addr)?
                            .to_string(),
                        msg: to_binary(&FundTransferMsgs::TransferFrom(TransferFromMsg {
                            owner: owner.clone(),
                            recipient: address.clone(),
                            amount: cw20_fund.amount,
                        }))?,
                        funds: vec![],
                    },
                    Fund::Cw721(cw721_fund) => WasmMsg::Execute {
                        contract_addr: deps
                            .api
                            .addr_validate(&cw721_fund.contract_addr)?
                            .to_string(),
                        msg: to_binary(&FundTransferMsgs::TransferNft(TransferNftMsg {
                            recipient: address.clone(),
                            token_id: cw721_fund.token_id.clone(),
                        }))?,
                        funds: vec![],
                    },
                }))
            }

            if let Some(msgs) = account_msgs {
                for msg in msgs {
                    msgs_vec.push(msg);
                }
            }

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
                .add_attribute("account_address", address)
                .add_attribute("funds", serde_json_wasm::to_string(&funds)?)
                .add_attribute("cw_funds", serde_json_wasm::to_string(&cw_funds_vec)?)
                .add_messages(msgs_vec))
        }
        // Job execution
        _ => {
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

            let config = CONFIG.load(deps.storage)?;

            // Assume reward.amount == warp token allowance
            let fee = finished_job.reward * Uint128::from(config.creation_fee_percentage)
                / Uint128::new(100);

            let account_amount = deps
                .querier
                .query::<BalanceResponse>(&QueryRequest::Bank(BankQuery::Balance {
                    address: finished_job.account.to_string(),
                    denom: config.fee_denom.clone(),
                }))?
                .amount
                .amount;

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
                                new_job_attrs.push(Attribute::new(
                                    "job_terminate_condition_status",
                                    "invalid",
                                ));
                                new_job_attrs.push(Attribute::new(
                                    "creation_status",
                                    format!(
                                        "terminated_due_to_terminate_condition_resolves_to_error. {}",
                                        e
                                    ),
                                ));
                            } else {
                                new_job_attrs.push(Attribute::new(
                                    "job_terminate_condition_status",
                                    "valid",
                                ));
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
                                terminate_condition: finished_job.terminate_condition.clone(),
                                vars: new_vars,
                                requeue_on_evict: finished_job.requeue_on_evict,
                                recurring: finished_job.recurring,
                                executions: finished_job.executions.clone(),
                                reward: finished_job.reward,
                                assets_to_withdraw: finished_job.assets_to_withdraw,
                            },
                        )?;

                        msgs.push(
                            // Job owner's warp account sends fee to fee collector
                            WasmMsg::Execute {
                                contract_addr: finished_job.account.to_string(),
                                msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {
                                    msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                                        to_address: config.fee_collector.to_string(),
                                        amount: vec![Coin::new(
                                            (fee).u128(),
                                            config.fee_denom.clone(),
                                        )],
                                    })],
                                }))?,
                                funds: vec![],
                            },
                        );

                        msgs.push(
                            // Job owner's warp account sends reward to controller
                            WasmMsg::Execute {
                                contract_addr: finished_job.account.to_string(),
                                msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {
                                    msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                                        to_address: env.contract.address.to_string(),
                                        amount: vec![Coin::new(
                                            (new_job.reward).u128(),
                                            config.fee_denom,
                                        )],
                                    })],
                                }))?,
                                funds: vec![],
                            },
                        );

                        msgs.push(
                            // Job owner withdraw all assets that are listed from warp account to itself
                            WasmMsg::Execute {
                                contract_addr: finished_job.account.to_string(),
                                msg: to_binary(&account::ExecuteMsg::WithdrawAssets(
                                    WithdrawAssetsMsg {
                                        asset_infos: new_job.assets_to_withdraw,
                                    },
                                ))?,
                                funds: vec![],
                            },
                        );

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

            Ok(Response::new()
                .add_attribute("action", "execute_reply")
                .add_attribute("job_id", finished_job.id)
                .add_attributes(res_attrs)
                .add_attributes(new_job_attrs)
                .add_messages(msgs))
        }
    }
}

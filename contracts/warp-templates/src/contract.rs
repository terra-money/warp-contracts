use crate::state::{CONFIG, QUERY_PAGE_SIZE, STATE, TEMPLATES};
use crate::ContractError;
use cosmwasm_std::{
    entry_point, to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Order, Response, StdError, StdResult, Uint64,
};
use cw_storage_plus::Bound;

use templates::template::{
    DeleteTemplateMsg, EditTemplateMsg, QueryTemplateMsg, QueryTemplatesMsg, SubmitTemplateMsg,
    Template, TemplateResponse, TemplatesResponse,
};
use templates::{
    Config, ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryConfigMsg, QueryMsg,
    State, UpdateConfigMsg,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        owner: deps.api.addr_validate(&msg.owner)?,
        fee_denom: msg.fee_denom,
        template_fee: Default::default(),
        fee_collector: deps.api.addr_validate(&msg.owner)?,
    };

    CONFIG.save(deps.storage, &config)?;

    let mut state = State {
        current_template_id: Default::default(),
    };

    for template in msg.templates {
        TEMPLATES.save(deps.storage, state.current_template_id.u64(), &template)?;
        state.current_template_id = state.current_template_id.checked_add(Uint64::one())?;
    }

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", config.owner)
        .add_attribute("template_fee", config.template_fee)
        .add_attribute("fee_collector", config.fee_collector)
        .add_attribute("current_template_id", state.current_template_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SubmitTemplate(data) => submit_template(deps, env, info, data),
        ExecuteMsg::EditTemplate(data) => edit_template(deps, env, info, data),
        ExecuteMsg::DeleteTemplate(data) => delete_template(deps, env, info, data),

        ExecuteMsg::UpdateConfig(data) => update_config(deps, env, info, data),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryTemplate(data) => to_binary(&query_template(deps, env, data)?),
        QueryMsg::QueryTemplates(data) => to_binary(&query_templates(deps, env, data)?),
        QueryMsg::QueryConfig(data) => to_binary(&query_config(deps, env, data)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn submit_template(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data: SubmitTemplateMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if !info.funds.contains(&Coin {
        denom: config.fee_denom.clone(),
        amount: config.template_fee,
    }) {
        return Err(ContractError::TemplateFeeNotFound {});
    }

    if data.name.len() > 280 {
        return Err(ContractError::NameTooLong {});
    }

    if data.name.is_empty() {
        return Err(ContractError::NameTooShort {});
    }

    if data.formatted_str.len() > 280 {
        return Err(ContractError::NameTooLong {});
    }

    if data.formatted_str.is_empty() {
        return Err(ContractError::NameTooShort {});
    }

    let state = STATE.load(deps.storage)?;
    let msg_template = Template {
        id: state.current_template_id,
        owner: info.sender.clone(),
        name: data.name.clone(),
        msg: data.msg.clone(),
        formatted_str: data.formatted_str.clone(),
        vars: data.vars.clone(),
        condition: data.condition.clone(),
    };

    TEMPLATES.save(deps.storage, state.current_template_id.u64(), &msg_template)?;
    STATE.save(
        deps.storage,
        &State {
            current_template_id: state
                .current_template_id
                .checked_add(Uint64::new(1))
                .map_err(|e| ContractError::CustomError { val: e.to_string() })?,
        },
    )?;

    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: config.fee_collector.to_string(),
        amount: vec![Coin::new((config.template_fee).u128(), config.fee_denom)],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "submit_msg_template")
        .add_attribute("id", state.current_template_id)
        .add_attribute("owner", info.sender)
        .add_attribute("name", data.name)
        .add_attribute("msg", data.msg)
        .add_attribute("formatted_str", data.formatted_str)
        .add_attribute("vars", serde_json_wasm::to_string(&data.vars)?))
}

pub fn edit_template(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data: EditTemplateMsg,
) -> Result<Response, ContractError> {
    let template = TEMPLATES.load(deps.storage, data.id.u64())?;

    if info.sender != template.owner {
        return Err(ContractError::Unauthorized {});
    }

    if data.name.is_some() && data.clone().name.unwrap().len() > 280 {
        return Err(ContractError::NameTooLong {});
    }

    if data.name.is_some() && data.name.clone().unwrap().is_empty() {
        return Err(ContractError::NameTooShort {});
    }

    let t = TEMPLATES.update(deps.storage, data.id.u64(), |t| match t {
        None => Err(ContractError::TemplateDoesNotExist {}),
        Some(t) => Ok(Template {
            id: t.id,
            owner: t.owner,
            name: data.name.unwrap_or(t.name),
            msg: t.msg,
            formatted_str: t.formatted_str,
            vars: t.vars,
            condition: t.condition,
        }),
    })?;

    Ok(Response::new()
        .add_attribute("action", "submit_msg_template")
        .add_attribute("id", t.id)
        .add_attribute("owner", info.sender)
        .add_attribute("name", t.name)
        .add_attribute("msg", t.msg)
        .add_attribute("formatted_str", t.formatted_str)
        .add_attribute("vars", serde_json_wasm::to_string(&t.vars)?))
}

pub fn delete_template(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data: DeleteTemplateMsg,
) -> Result<Response, ContractError> {
    let template = TEMPLATES.load(deps.storage, data.id.u64())?;

    if info.sender != template.owner {
        return Err(ContractError::Unauthorized {});
    }

    TEMPLATES.remove(deps.storage, data.id.u64());

    Ok(Response::new()
        .add_attribute("action", "delete_template")
        .add_attribute("id", data.id))
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    config.owner = match data.owner {
        None => config.owner,
        Some(data) => deps.api.addr_validate(data.as_str())?,
    };

    config.fee_denom = match data.fee_denom {
        None => config.fee_denom,
        Some(data) => data,
    };

    config.fee_collector = match data.fee_collector {
        None => config.fee_collector,
        Some(data) => deps.api.addr_validate(data.as_str())?,
    };
    config.template_fee = data.template_fee.unwrap_or(config.template_fee);

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("owner", config.owner)
        .add_attribute("template_fee", config.template_fee)
        .add_attribute("fee_collector", config.fee_collector))
}

pub fn query_template(
    deps: Deps,
    _env: Env,
    data: QueryTemplateMsg,
) -> StdResult<TemplateResponse> {
    let msg_template = TEMPLATES.load(deps.storage, data.id.u64())?;
    Ok(TemplateResponse {
        template: msg_template,
    })
}

pub fn query_templates(
    deps: Deps,
    env: Env,
    data: QueryTemplatesMsg,
) -> StdResult<TemplatesResponse> {
    if !data.valid_query() {
        return Err(StdError::generic_err(
            "Invalid query input. Must supply at most one of ids, name, or owner params.",
        ));
    }

    let _page_size = data.limit.unwrap_or(QUERY_PAGE_SIZE);

    match data {
        QueryTemplatesMsg { ids: Some(ids), .. } => {
            if ids.len() > QUERY_PAGE_SIZE as usize {
                return Err(StdError::generic_err(
                    "Number of ids supplied exceeds query limit",
                ));
            }

            let mut msg_templates = vec![];

            for id in ids {
                let msg_template =
                    query_template(deps, env.clone(), QueryTemplateMsg { id })?.template;
                msg_templates.push(msg_template);
            }
            Ok(TemplatesResponse {
                templates: msg_templates,
            })
        }
        QueryTemplatesMsg {
            start_after,
            limit,
            name,
            owner,
            ..
        } => {
            let start = start_after.map(Bound::exclusive);

            let infos = TEMPLATES
                .range(deps.storage, start, None, Order::Ascending)
                .filter(|m| {
                    (name.is_none() || name.clone().unwrap() == m.as_ref().unwrap().clone().1.name)
                        && (owner.is_none()
                            || owner.clone().unwrap() == m.as_ref().unwrap().clone().1.owner)
                });
            let infos = match limit {
                None => infos
                    .take(QUERY_PAGE_SIZE as usize)
                    .collect::<StdResult<Vec<_>>>()?,
                Some(limit) => infos.take(limit as usize).collect::<StdResult<Vec<_>>>()?,
            };
            let mut msg_templates = vec![];
            for info in infos {
                msg_templates.push(info.1);
            }
            Ok(TemplatesResponse {
                templates: msg_templates,
            })
        }
    }
}

pub fn query_config(deps: Deps, _env: Env, _data: QueryConfigMsg) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

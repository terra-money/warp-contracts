use crate::state::{QUERY_PAGE_SIZE, TEMPLATES};
use cosmwasm_std::{Deps, Env, Order, StdError, StdResult};
use cw_storage_plus::Bound;

use warp_protocol::controller::template::{
    QueryTemplateMsg, QueryTemplatesMsg, TemplateResponse, TemplatesResponse,
};

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
    //todo: separate code into fns
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


use crate::state::{MSG_TEMPLATES, QUERY_PAGE_SIZE};
use cosmwasm_std::{Deps, Env, MessageInfo, Order, StdError, StdResult};
use cw_storage_plus::Bound;

use warp_protocol::controller::template::{
    MsgTemplateResponse, MsgTemplatesResponse, QueryMsgTemplateMsg,
    QueryMsgTemplatesMsg,
};

pub fn query_msg_template(
    deps: Deps,
    _env: Env,
    _info: MessageInfo,
    data: QueryMsgTemplateMsg,
) -> StdResult<MsgTemplateResponse> {
    let msg_template = MSG_TEMPLATES.load(deps.storage, data.id.u64())?;
    Ok(MsgTemplateResponse {
        template: msg_template,
    })
}

pub fn query_msg_templates(
    //todo: separate code into fns
    deps: Deps,
    env: Env,
    info: MessageInfo,
    data: QueryMsgTemplatesMsg,
) -> StdResult<MsgTemplatesResponse> {
    if !data.valid_query() {
        return Err(StdError::generic_err(
            "Invalid query input. Must supply at most one of ids, name, or owner params.",
        ));
    }

    let _page_size = data.limit.unwrap_or(QUERY_PAGE_SIZE);

    match data {
        QueryMsgTemplatesMsg { ids: Some(ids), .. } => {
            if ids.len() > QUERY_PAGE_SIZE as usize {
                return Err(StdError::generic_err(
                    "Number of ids supplied exceeds query limit",
                ));
            }

            let mut msg_templates = vec![];

            for id in ids {
                let msg_template = query_msg_template(
                    deps,
                    env.clone(),
                    info.clone(),
                    QueryMsgTemplateMsg { id },
                )?
                .template;
                msg_templates.push(msg_template);
            }
            return Ok(MsgTemplatesResponse {
                templates: msg_templates,
            });
        }
        QueryMsgTemplatesMsg {
            start_after,
            limit,
            name,
            owner,
            ..
        } => {
            let start = start_after.map(Bound::exclusive);

            let infos = MSG_TEMPLATES
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
            return Ok(MsgTemplatesResponse {
                templates: msg_templates,
            });
        }
    }
}

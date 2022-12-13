use crate::state::{MSG_TEMPLATES, STATE};
use crate::ContractError;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint64};
use warp_protocol::controller::controller::State;
use warp_protocol::controller::template::{
    DeleteTemplateMsg, EditTemplateMsg, Template, SubmitTemplateMsg,
};

pub fn submit_msg_template(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data: SubmitTemplateMsg,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    let msg_template = Template {
        id: state.current_template_id,
        owner: info.sender.clone(),
        name: data.name.clone(),
        kind: data.kind.clone(),
        msg: data.msg.clone(),
        formatted_str: data.formatted_str.clone(),
        vars: data.vars.clone(),
    };
    MSG_TEMPLATES.save(deps.storage, state.current_template_id.u64(), &msg_template)?;
    STATE.save(
        deps.storage,
        &State {
            current_job_id: state.current_job_id,
            current_template_id: state.current_template_id.saturating_add(Uint64::new(1)),
        },
    )?;
    Ok(Response::new()
        .add_attribute("action", "submit_msg_template")
        .add_attribute("id", state.current_template_id)
        .add_attribute("owner", info.sender)
        .add_attribute("name", data.name)
        .add_attribute("kind", serde_json_wasm::to_string(&data.kind)?)
        .add_attribute("msg", data.msg)
        .add_attribute("formatted_str", data.formatted_str)
        .add_attribute("vars", serde_json_wasm::to_string(&data.vars)?))
}

pub fn edit_msg_template(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _data: EditTemplateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn delete_msg_template(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _data: DeleteTemplateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

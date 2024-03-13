use account_tracker::UpdateConfigMsg;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{state::CONFIG, ContractError};

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    config.admin = match data.admin {
        None => config.admin,
        Some(data) => deps.api.addr_validate(data.as_str())?,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("config_admin", config.admin))
}

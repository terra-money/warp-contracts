use crate::state::CONFIG;
use crate::ContractError;
use controller::UpdateConfigMsg;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

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

    config.fee_collector = match data.fee_collector {
        None => config.fee_collector,
        Some(data) => deps.api.addr_validate(data.as_str())?,
    };
    config.minimum_reward = data.minimum_reward.unwrap_or(config.minimum_reward);
    config.creation_fee_percentage = data
        .creation_fee_percentage
        .unwrap_or(config.creation_fee_percentage);
    config.cancellation_fee_percentage = data
        .cancellation_fee_percentage
        .unwrap_or(config.cancellation_fee_percentage);

    config.template_fee = data.template_fee.unwrap_or(config.template_fee);

    config.a_max = data.a_max.unwrap_or(config.a_max);
    config.a_min = data.a_min.unwrap_or(config.a_min);
    config.t_max = data.t_max.unwrap_or(config.t_max);
    config.t_min = data.t_min.unwrap_or(config.t_min);
    config.q_max = data.q_max.unwrap_or(config.q_max);

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

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("config_owner", config.owner)
        .add_attribute("config_minimum_reward", config.minimum_reward)
        .add_attribute(
            "config_creation_fee_percentage",
            config.creation_fee_percentage,
        )
        .add_attribute(
            "config_cancellation_fee_percentage",
            config.cancellation_fee_percentage,
        )
        .add_attribute("config_template_fee", config.template_fee)
        .add_attribute("config_a_max", config.a_max)
        .add_attribute("config_a_min", config.a_min)
        .add_attribute("config_t_max", config.t_max)
        .add_attribute("config_t_min", config.t_min)
        .add_attribute("config_q_max", config.q_max))
}

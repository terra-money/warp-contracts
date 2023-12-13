use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{state::CONFIG, ContractError};

use controller::{Config, UpdateConfigMsg};

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data: UpdateConfigMsg,
    mut config: Config,
) -> Result<Response, ContractError> {
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
    config.cancellation_fee_rate = data
        .cancellation_fee_rate
        .unwrap_or(config.cancellation_fee_rate);

    config.creation_fee_min = data.creation_fee_min.unwrap_or(config.creation_fee_min);
    config.creation_fee_max = data.creation_fee_max.unwrap_or(config.creation_fee_max);
    config.burn_fee_min = data.burn_fee_min.unwrap_or(config.burn_fee_min);
    config.maintenance_fee_min = data
        .maintenance_fee_min
        .unwrap_or(config.maintenance_fee_min);
    config.maintenance_fee_max = data
        .maintenance_fee_max
        .unwrap_or(config.maintenance_fee_max);
    config.duration_days_min = data.duration_days_min.unwrap_or(config.duration_days_min);
    config.duration_days_max = data.duration_days_max.unwrap_or(config.duration_days_max);
    config.queue_size_left = data.queue_size_left.unwrap_or(config.queue_size_left);
    config.queue_size_right = data.queue_size_right.unwrap_or(config.queue_size_right);
    config.burn_fee_rate = data.burn_fee_rate.unwrap_or(config.burn_fee_rate);

    if config.burn_fee_rate.u128() > 100 {
        return Err(ContractError::BurnFeeTooHigh {});
    }

    if config.creation_fee_max < config.creation_fee_min {
        return Err(ContractError::CreationMaxFeeUnderMinFee {});
    }

    if config.maintenance_fee_max < config.maintenance_fee_min {
        return Err(ContractError::MaintenanceMaxFeeUnderMinFee {});
    }

    if config.duration_days_max < config.duration_days_min {
        return Err(ContractError::DurationMaxDaysUnderMinDays {});
    }

    if config.cancellation_fee_rate.u64() > 100 {
        return Err(ContractError::CancellationFeeTooHigh {});
    }

    if config.burn_fee_rate.u128() > 100 {
        return Err(ContractError::BurnFeeTooHigh {});
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("config_owner", config.owner)
        .add_attribute("config_fee_collector", config.fee_collector)
        .add_attribute("config_minimum_reward", config.minimum_reward)
        .add_attribute("config_cancellation_fee_rate", config.cancellation_fee_rate))
}

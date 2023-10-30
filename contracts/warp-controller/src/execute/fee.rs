use controller::Config;
use cosmwasm_std::Uint128;

pub fn compute_creation_fee(queue_size: Uint128, config: &Config) -> Uint128 {
    let x1 = config.queue_size_left;
    let y1 = config.creation_fee_min;
    let x2 = config.queue_size_right;
    let y2 = config.creation_fee_max;

    let slope = (y2 - y1) / (x2 - x1);

    if queue_size < x1 {
        config.creation_fee_min
    } else if queue_size < x2 {
        slope * queue_size + y1 - slope * x1
    } else {
        config.creation_fee_max
    }
}

pub fn compute_maintenance_fee(duration_days: Uint128, config: &Config) -> Uint128 {
    let x1 = config.duration_days_left;
    let y1 = config.maintenance_fee_min;
    let x2 = config.duration_days_right;
    let y2 = config.maintenance_fee_max;

    let slope = (y2 - y1) / (x2 - x1);

    if duration_days < x1 {
        config.maintenance_fee_min
    } else if duration_days < x2 {
        slope * duration_days + y1 - slope * x1
    } else {
        config.maintenance_fee_max
    }
}

pub fn compute_burn_fee(job_reward: Uint128, config: &Config) -> Uint128 {
    let min_fee: Uint128 = config.burn_fee_min;
    let calculated_fee = job_reward * config.burn_fee_rate / Uint128::new(100);

    if calculated_fee > min_fee {
        calculated_fee
    } else {
        min_fee
    }
}

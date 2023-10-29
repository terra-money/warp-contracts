use controller::Config;
use cosmwasm_std::Uint128;

const FIXED_POINT_FACTOR: Uint128 = Uint128::new(1_000_000u128);

fn sigmoid(x: Uint128) -> Uint128 {
    // Sigmoid function: 1 / (1 + exp(-x))
    let one = FIXED_POINT_FACTOR;

    // Using the negative exponentiation rule: exp(-x) = 1 / exp(x)
    let exp_neg_x = one / exp(x);
    one / (one + exp_neg_x)
}

fn exp(x: Uint128) -> Uint128 {
    // For simplicity, we are using the exponential function's Taylor series expansion:
    // exp(x) = 1 + x + x^2/2! + x^3/3! + ...
    let mut result = FIXED_POINT_FACTOR;
    let mut term = FIXED_POINT_FACTOR;

    for n in 1..10 {
        term = term * x / Uint128::new(n as u128);
        result += term;
    }

    result
}

fn smooth_transition(x: Uint128, min: Uint128, max: Uint128) -> Uint128 {
    const K_CONSTANT_FACTOR: Uint128 = Uint128::new(2);
    const K_DIVISOR: Uint128 = Uint128::new(10);

    let a = min;
    let b = max;
    let c = (max + min) / Uint128::new(2);

    let k_constant = K_CONSTANT_FACTOR * (FIXED_POINT_FACTOR / K_DIVISOR);
    let sigmoid_val = sigmoid((k_constant * (x - c)) / Uint128::from(FIXED_POINT_FACTOR));

    a + ((b - a) * sigmoid_val) / Uint128::from(FIXED_POINT_FACTOR)
}

// can be in native decimals
pub fn compute_creation_fee(queue_size: Uint128, config: &Config) -> Uint128 {
    let x1 = config.queue_size_left;
    let y1 = config.creation_fee_min;
    let x2 = config.queue_size_right;
    let y2 = config.creation_fee_max;

    let slope = (y2 - y1) / (x2 - x1);
    let y_intercept = y1 - slope * x1;

    if queue_size < x1 {
        config.creation_fee_min
    } else if queue_size < x2 {
        slope * queue_size + y_intercept
    } else {
        config.creation_fee_max
    }
}

// can be in native decimals
pub fn compute_maintenance_fee(duration_days: Uint128, config: &Config) -> Uint128 {
    if duration_days < config.duration_days_left {
        config.maintenance_fee_min
    } else if duration_days <= config.duration_days_right {
        smooth_transition(
            duration_days,
            config.maintenance_fee_min,
            config.maintenance_fee_max,
        )
    } else {
        config.maintenance_fee_max
    }
}

// can be in native decimals
pub fn compute_burn_fee(job_reward: Uint128, config: &Config) -> Uint128 {
    let min_fee = config.burn_fee_min;
    let calculated_fee = job_reward * config.burn_fee_rate / Uint128::new(100);

    if calculated_fee > min_fee {
        calculated_fee
    } else {
        min_fee
    }
}

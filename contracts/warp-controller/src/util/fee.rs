use cosmwasm_std::{Coin, Uint128};

pub fn deduct_reward_and_fee_from_native_funds(
    funds: Vec<Coin>,
    fee_denom: String,
    reward_plus_fee: Uint128,
) -> Vec<Coin> {
    let mut funds = funds;
    let mut deducted_amount = reward_plus_fee;
    for fund in funds.iter_mut() {
        if fund.denom == fee_denom {
            fund.amount = fund.amount.checked_sub(deducted_amount).unwrap();
            deducted_amount = Uint128::zero();
        }
    }
    funds
}

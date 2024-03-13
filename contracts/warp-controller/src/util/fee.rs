use cosmwasm_std::{Coin, Uint128};

pub fn deduct_from_native_funds(
    funds: Vec<Coin>,
    fee_denom: String,
    deducted: Uint128,
) -> Vec<Coin> {
    let mut funds = funds;
    let mut deducted_amount = deducted;
    for fund in funds.iter_mut() {
        if fund.denom == fee_denom {
            fund.amount = fund.amount.checked_sub(deducted_amount).unwrap();
            deducted_amount = Uint128::zero();
        }
    }

    // Filter out coins with an amount of zero
    funds.retain(|coin| !coin.amount.is_zero());

    funds
}

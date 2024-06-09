use cosmwasm_std::{to_binary, Deps, Uint128};

use crate::state::FEE;

pub fn create_key_hash(con_1: String, con_2: String) -> String {
    to_binary(&format!("{}{}", con_1, con_2))
        .unwrap()
        .to_string()
}

pub fn cal_min_required(prize_pool: Uint128, fee: Uint128) -> Uint128 {
    prize_pool
        .checked_sub(fee * Uint128::new(2))
        .unwrap()
        .checked_div_euclid(Uint128::new(2))
        .unwrap()
}

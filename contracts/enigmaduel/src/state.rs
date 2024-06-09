use cosmwasm_schema::cw_serde;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, DepsMut, Uint128};
use cw_storage_plus::{Item, Map};

use crate::msg::GameRoomStatus;

#[cw_serde]
pub struct GameRoomsState {
    pub contestant1: String,
    pub contestant2: String,
    pub prize_pool: Uint128,
    pub status: GameRoomStatus,
}

impl GameRoomsState {
    pub fn get_finish_state(&self, status: GameRoomStatus) -> Self {
        Self {
            contestant1: self.contestant1.clone(),
            contestant2: self.contestant2.clone(),
            prize_pool: Default::default(),
            status,
        }
    }
}
#[cw_serde]
pub struct Balance {
    pub total: Uint128,
    pub locked: Uint128,
}

impl Balance {
    pub fn new_zero() -> Self {
        Self {
            total: Uint128::zero(),
            locked: Uint128::zero(),
        }
    }

    pub fn total_increase(self, amount: Uint128) -> Self {
        Self {
            total: self.total.checked_add(amount).unwrap(),
            locked: self.locked,
        }
    }

    pub fn total_decrease(self, amount: Uint128) -> Self {
        Self {
            total: self.total.checked_sub(amount).unwrap(),
            locked: self.locked,
        }
    }

    pub fn lock(&mut self, amount: Uint128) -> Self {
        Self {
            total: self.total.checked_sub(amount).unwrap(),
            locked: self.locked.checked_add(amount).unwrap(),
        }
    }

    pub fn unlock_and_increase(
        &mut self,
        unlock_amount: Uint128,
        increase_amount: Uint128,
    ) -> Self {
        Self {
            total: self
                .total
                .checked_add(unlock_amount)
                .unwrap()
                .checked_add(increase_amount)
                .unwrap(),
            locked: self.locked.checked_sub(unlock_amount).unwrap(),
        }
    }

    pub fn unlock_and_decrease(
        &mut self,
        unlock_amount: Uint128,
        decrease_amount: Uint128,
    ) -> Self {
        Self {
            total: self
                .total
                .checked_add(unlock_amount)
                .unwrap()
                .checked_sub(decrease_amount)
                .unwrap(),
            locked: self.locked.checked_sub(unlock_amount).unwrap(),
        }
    }

    pub fn available_balance(self) -> Uint128 {
        self.total.checked_sub(self.locked).unwrap()
    }
}

impl Default for Balance {
    fn default() -> Self {
        Self {
            total: Default::default(),
            locked: Default::default(),
        }
    }
}

pub const ADMIN: Item<Addr> = Item::new("admin");
pub const FEE: Item<Uint128> = Item::new("fee");
// the admin address will be saved in this mapping for ease of use, the balance will be modified after each change accordingly to the fee amount that is reduced.
pub const BALANCES: Map<&Addr, Balance> = Map::new("balance");

pub const GAME_ROOMS_STATE: Map<String, GameRoomsState> = Map::new("game_rooms");
pub const ENIGMA_DUEL_TOKEN: Item<Addr> = Item::new("enigma_duel_token");

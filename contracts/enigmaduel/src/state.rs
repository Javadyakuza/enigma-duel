use cosmwasm_schema::cw_serde;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, DepsMut, Uint128};
use cw_storage_plus::{Item, Map};

use crate::msg::GameRoomStatus;

#[cw_serde]
pub struct Fee {
    pub draw: Uint128,
    pub win: Uint128,
}

#[cw_serde]
pub struct GameRoomsState {
    pub contestant1: String,
    pub contestant2: String,
    pub prize_pool: Uint128,
    pub status: GameRoomStatus,
}

pub fn new(&mut self, deps: DepsMut, withdraw_addr: String, balance: Uint128) -> Self {
    STR_TO_INT_MAP.save(deps.storage, "ten".to_owned(), 10);
    STR_TO_INT_MAP.save(deps.storage, "one".to_owned(), 1);

    let ten = STR_TO_INT_MAP.load(deps.storage, "ten".to_owned())?;
    assert_eq!(ten, 10);

    let two = STR_TO_INT_MAP.may_load(deps.storage, "two".to_owned())?;
    assert_eq!(two, None);
}

pub const ADMIN: Item<Addr> = Item::new("admin");
pub const FEE: Item<Fee> = Item::new("fee");
// the admin address will be saved in this mapping for ease of use, the balance will be modified after each change accordingly to the fee amount that is reduced.
pub const BALANCES: Map<&Addr, Uint128> = Map::new("balance");
pub const GAME_ROOMS_STATE: Map<i64, GameRoomsState> = Map::new("game_rooms");

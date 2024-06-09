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

#[cw_serde]
pub struct Balance {
    total: Uint128,
    locked: Uint128,
}

pub const ADMIN: Item<Addr> = Item::new("admin");
pub const FEE: Item<Uint128> = Item::new("fee");
// the admin address will be saved in this mapping for ease of use, the balance will be modified after each change accordingly to the fee amount that is reduced.
pub const BALANCES: Map<&Addr, Balance> = Map::new("balance");

pub const GAME_ROOMS_STATE: Map<String, GameRoomsState> = Map::new("game_rooms");
pub const ENIGMA_DUEL_TOKEN: Item<Addr> = Item::new("enigma_duel_token");

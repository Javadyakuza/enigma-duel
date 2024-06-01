use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, DepsMut, Uint128};

use crate::state::Fee;

// todo !
#[cw_serde]
pub struct InstantiateMsg {
    pub fee: Fee,
    pub admin: String,
    pub enigma_token_duel: String,
}

// executing input and output structs/enums //

// structures
#[cw_serde]
pub struct GameRoomIntiParams {
    pub contestant1: String,
    pub contestant2: String,
    // the whole prize pool amount + Enigma Fee.
    pub prize_pool: Uint128,
    pub status: GameRoomStatus,
}

#[cw_serde]
pub struct GameRoomFinishParams {
    pub game_room_id: i64,
    pub Result: GameRoomStatus,
}

#[cw_serde]
pub enum GameRoomStatus {
    Win { winner_addr: String },
    Draw {},
    OnGoing {},
}
#[cw_serde]
pub struct SendFrom {
    pub owner: String,
    pub contract: String,
    pub amount: Uint128,
    pub msg: Binary,
}

#[cw_serde]
pub enum UpdateBalanceMode {
    Deposit { amount: Uint128 },
    Withdraw { amount: Uint128, receiver: String },
}
// input messages
#[cw_serde]
pub enum ExecuteMsg {
    // Deposit
    UpdateBalance {
        update_mode: UpdateBalanceMode,
    },

    CreateGameRoom {
        game_room_init_params: GameRoomIntiParams,
    },
    FinishGameRoom {
        game_room_id: String,
    }, // no output considered for this instruction
    CollectFees {
        amount: Uint128,
    },
    Receive(Cw20ReceiveMsg),
}

// output structs

#[cw_serde]
pub struct BalanceChangeResp(pub Uint128); // the new balance.

#[cw_serde]
pub struct CreateGameRoomResp(pub i64); // the game room id.

// executing input and output structs/messages //

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetUserBalanceResp)]
    GetUserBalance { user: String },
    #[returns(GameRoomStatus)]
    GetGameRoomState { Game_room_id: i64 },
    #[returns(GetCollectedFeesResp)]
    GetCollectedFees {},
    // TVL is the contract balance
    #[returns(GetTotalGamesResp)]
    GetTotalGames {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetUserBalanceResp {
    pub balance: Uint128,
}
#[cw_serde]
pub struct GetCollectedFeesResp {
    pub fees: Uint128,
}
#[cw_serde]
pub struct GetTotalGamesResp {
    pub total_games: i64,
}

#[cw_serde]
pub struct Cw20ReceiveMsg {
    pub sender: String,
    pub amount: Uint128,
    pub msg: Binary,
}

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Uint128};

// todo !
#[cw_serde]
pub struct InstantiateMsg {
    pub fee: Uint128,
    pub admin: String,
    pub enigma_token_duel: String,
}

// executing input and output structs/enums //

// structures
#[cw_serde]
pub struct GameRoomIntiParams {
    pub contestant1: String,
    pub contestant2: String,
    // contestant one share + contestant two share + Enigma Duel Fee.
    pub prize_pool: Uint128,
    pub status: GameRoomStatus,
}

#[cw_serde]
pub struct GameRoomFinishParams {
    pub game_room_key: String,
    pub result: GameRoomStatus,
}

#[cw_serde]
pub struct CollectFeesParams {
    pub amount: Uint128,
    pub receiver: String,
}

#[cw_serde]
pub enum GameRoomStatus {
    Started {},
    Win { addr: String },
    Draw {},
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
    Deposit {
        user: Option<String>,
        amount: Uint128,
    },
    Withdraw {
        user: Option<String>,
        amount: Uint128,
        receiver: String,
    },
}

impl UpdateBalanceMode {
    pub fn to_string(&self) -> String {
        match self {
            Self::Deposit { user, amount } => {
                format!(
                    "Deposit {} tokens for user {} ",
                    amount,
                    user.clone().unwrap()
                )
            }
            Self::Withdraw {
                user,
                amount,
                receiver,
            } => {
                format!(
                    "withdraw {} tokens for user {} from {} ",
                    amount,
                    receiver,
                    user.clone().unwrap_or_default(),
                )
            }
        }
    }
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
        game_room_finish_params: GameRoomFinishParams,
    },
    CollectFees {
        collect_fees_params: CollectFeesParams,
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
    #[returns(GetUserBalanceResp)]
    GetUserLockedBalance { user: String },
    #[returns(GameRoomStatus)]
    GetGameRoomState { game_room_key: String },
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

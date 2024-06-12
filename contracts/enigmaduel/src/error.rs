use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("insufficient Balance")]
    InsufficientBalance(InsufficientBalanceErr),

    #[error("Game room is not started yet")]
    GameRoomNotStarted {},

    #[error("Game room already started")]
    GameRoomAlreadyStarted {},

    #[error("Game room load error")]
    GameRoomLoadError { msg: String },
}

#[cw_serde]
pub struct InsufficientBalanceErr {
    pub min_required: Uint128,
    pub current_balance: Uint128,
    pub user: String,
}

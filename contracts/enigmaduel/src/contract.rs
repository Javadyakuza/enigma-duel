#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
    Uint256,
};
use cw2::set_contract_version;
use execute::*;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GameRoomStatus, InstantiateMsg, QueryMsg};
use crate::state::{
    Balance, GameRoomsState, ADMIN, BALANCES, ENIGMA_DUEL_TOKEN, FEE, GAME_ROOMS_COUNT,
    GAME_ROOMS_STATE,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enigmaduel";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // setting the contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // setting the fee for each game result
    FEE.save(deps.storage, &msg.fee)?;

    // setting the admin address that collects the fees as well.
    ADMIN.save(deps.storage, &Addr::unchecked(msg.admin.clone()))?;

    // instantiating the admin address as the fee collector.
    BALANCES.save(
        deps.storage,
        &Addr::unchecked(msg.admin.clone()),
        &Balance::new_zero(),
    )?;

    // instantiating the enigma duel token address.
    ENIGMA_DUEL_TOKEN.save(deps.storage, &(Addr::unchecked(msg.enigma_token_duel)))?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", msg.admin)
        .add_attribute("fee", msg.fee))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateBalance { update_mode } => {
            execute::update_balance(deps, _env, info, update_mode)
        }
        ExecuteMsg::CreateGameRoom {
            game_room_init_params,
        } => create_game_room(deps, info, game_room_init_params),
        ExecuteMsg::FinishGameRoom {
            game_room_finish_params,
        } => finish_game_room(deps, info, game_room_finish_params),
        ExecuteMsg::CollectFees {
            collect_fees_params,
        } => collect_fees(deps, info, collect_fees_params),
        ExecuteMsg::Receive(receive_msg) => {
            execute::update_balance_callback(deps, info, receive_msg.msg)
        }
    }
}

pub mod execute {
    use cosmwasm_std::{from_json, Uint256};

    use super::*;
    use crate::{
        error::{self, InsufficientBalanceErr},
        helpers::{cal_min_required, create_key_hash},
        msg::{
            CollectFeesParams, GameRoomFinishParams, GameRoomIntiParams,
            UpdateBalanceMode::{self, *},
        },
        state::GAME_ROOMS_COUNT,
    };

    // creating a proper response for each function
    pub fn update_balance(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        update_mode: UpdateBalanceMode,
    ) -> Result<Response, ContractError> {
        // address doesn't need be validated because the internal state is not getting changed,
        // in the call back we change the balance and we are sure that the address is correct.
        // Your contract logic here
        Ok(Response::new()
            .add_attribute("action", "update_balance_request")
            .add_attribute("request_data", update_mode.to_string())
            .add_message(
                // matching the mode
                match update_mode {
                    Deposit { amount, .. } => cosmwasm_std::WasmMsg::Execute {
                        contract_addr: ENIGMA_DUEL_TOKEN.load(deps.storage)?.into(),
                        msg: to_json_binary(&cw20::Cw20ExecuteMsg::SendFrom {
                            owner: info.sender.clone().into(),
                            contract: env.contract.address.into(),
                            amount,
                            msg: to_json_binary(&Deposit {
                                user: Some(info.sender.into()),
                                amount,
                            })?,
                        })?,
                        funds: vec![],
                    },
                    Withdraw {
                        amount, receiver, ..
                    } => {
                        // checking the balance

                        if let Ok(balance) =
                            BALANCES.load(deps.storage, &Addr::unchecked(receiver.clone()))
                        {
                            if balance.total < amount {
                                return Err(error::ContractError::InsufficientBalance(
                                    InsufficientBalanceErr {
                                        min_required: amount,
                                        current_balance: balance.available_balance(),
                                        user: receiver.clone(),
                                    },
                                ));
                            }
                        };

                        BALANCES.update(
                            deps.storage,
                            &Addr::unchecked(receiver.clone()),
                            |balance: Option<Balance>| -> StdResult<_> {
                                Ok(balance.unwrap_or_default().total_decrease(amount))
                            },
                        )?;

                        cosmwasm_std::WasmMsg::Execute {
                            contract_addr: ENIGMA_DUEL_TOKEN.load(deps.storage)?.into(),
                            msg: to_json_binary(&cw20::Cw20ExecuteMsg::Transfer {
                                recipient: receiver.clone(),
                                amount,
                            })?,
                            funds: vec![],
                        }
                    }
                },
            ))
    }

    pub fn update_balance_callback(
        deps: DepsMut,
        info: MessageInfo,
        update_mode: Binary,
    ) -> Result<Response, ContractError> {
        let edt_addr: Addr = ENIGMA_DUEL_TOKEN.load(deps.storage)?;

        if info.sender != edt_addr {
            return Err(error::ContractError::Unauthorized {});
        }

        let update_balance_data: UpdateBalanceMode =
            match from_json::<UpdateBalanceMode>(&update_mode)? {
                Deposit { amount, user } => {
                    BALANCES.update(
                        deps.storage,
                        &Addr::unchecked(user.clone().unwrap()),
                        |balance: Option<Balance>| -> StdResult<_> {
                            Ok(balance
                                .unwrap_or(Balance::new_zero())
                                .total_increase(amount))
                        },
                    )?;
                    Deposit { user, amount }
                }
                Withdraw {
                    amount,
                    user,
                    receiver,
                } => {
                    BALANCES.update(
                        deps.storage,
                        &Addr::unchecked(user.clone().unwrap()),
                        |balance: Option<Balance>| -> StdResult<_> {
                            Ok(balance.unwrap().total_decrease(amount))
                        },
                    )?;
                    Withdraw {
                        user,
                        amount,
                        receiver,
                    }
                }
            };
        Ok(Response::new()
            .add_attribute("action", "update_balance_confirmed")
            .add_attribute("update_balance_data", update_balance_data.to_string()))
    }

    pub fn create_game_room(
        deps: DepsMut,
        info: MessageInfo,
        params: GameRoomIntiParams,
    ) -> Result<Response, ContractError> {
        // sender must be app admin
        if info.sender != ADMIN.load(deps.storage)? {
            return Err(error::ContractError::Unauthorized {});
        }

        // each contestant must have whole prize pool amount - enigma duel fee / 2 token balances
        // loading the fee
        let min_required = cal_min_required(params.prize_pool, Uint128::zero());

        let (con_1_bal, con_2_bal) = (
            BALANCES
                .may_load(deps.storage, &Addr::unchecked(params.contestant1.clone()))
                .unwrap()
                .unwrap(),
            BALANCES
                .may_load(deps.storage, &Addr::unchecked(params.contestant2.clone()))
                .unwrap()
                .unwrap(),
        );
        println!("got the balances, {:?}{:?}", con_1_bal, con_2_bal);

        // in the following line we also check the prize pool to not be zero
        let con_1_av = con_1_bal.available_balance();
        let con_2_av = con_2_bal.available_balance();
        if min_required >= con_1_av || min_required >= con_2_av {
            return Err(error::ContractError::InsufficientBalance(
                InsufficientBalanceErr {
                    min_required,
                    current_balance: con_1_av,
                    user: params.contestant1.clone(),
                },
            ));
        } else if min_required >= con_2_av {
            return Err(error::ContractError::InsufficientBalance(
                InsufficientBalanceErr {
                    min_required,
                    current_balance: con_2_av,
                    user: params.contestant2.clone(),
                },
            ));
        }
        println!("got the balances");

        // creating the key of the these two components as the key
        let game_room_key = create_key_hash(params.contestant1.clone(), params.contestant2.clone());
        let game_room_data = GameRoomsState {
            contestant1: params.contestant1.clone(),
            contestant2: params.contestant2.clone(),
            prize_pool: params.prize_pool,
            status: GameRoomStatus::Started {},
        };
        println!("got the balances");

        // checking the previous existence
        match GAME_ROOMS_STATE.may_load(deps.storage, game_room_key.clone()) {
            // at this point the game room was initialized previously, we check that the game room must have been finished previously
            Ok(option_state) => match option_state {
                Some(state) => match state.status {
                    GameRoomStatus::Started {} => {
                        return Err(error::ContractError::GameRoomAlreadyStarted {});
                    }

                    _ =>
                    // exists before, updating
                    {
                        GAME_ROOMS_STATE.update(
                            deps.storage,
                            game_room_key.clone(),
                            |state: Option<GameRoomsState>| -> Result<GameRoomsState, ContractError> {
                                match state {
                                    Some(_) => Ok(game_room_data.clone()),
                                    None => Err(error::ContractError::GameRoomLoadError { msg: "couldn't load existing room !".to_string() }),
                                }
                            },
                        )?;
                    }
                },
                None => {
                    // doesn't exits adding
                    GAME_ROOMS_STATE.save(deps.storage, game_room_key.clone(), &game_room_data)?
                }
            },
            Err(e) => return Err(error::ContractError::GameRoomLoadError { msg: e.to_string() }),
        }

        // locking the prize pool amount form the both contestants
        println!("got the balances");

        // locking
        BALANCES.update(
            deps.storage,
            &Addr::unchecked(params.contestant1),
            |balance: Option<Balance>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().lock(min_required))
            },
        )?;

        // locking
        BALANCES.update(
            deps.storage,
            &Addr::unchecked(params.contestant2),
            |balance: Option<Balance>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().lock(min_required))
            },
        )?;

        let count = &GAME_ROOMS_COUNT
            .load(deps.storage)
            .unwrap_or(Uint256::zero());
        GAME_ROOMS_COUNT
            .save(deps.storage, &count.checked_add(Uint256::one()).unwrap())
            .unwrap();
        Ok(Response::new()
            .add_attribute("action", "crate_game_room")
            .add_attribute("room_key", game_room_key))
    }

    pub fn finish_game_room(
        deps: DepsMut,
        info: MessageInfo,
        params: GameRoomFinishParams,
    ) -> Result<Response, ContractError> {
        // sender must be app admin
        if info.sender != ADMIN.load(deps.storage)? {
            return Err(error::ContractError::Unauthorized {});
        }

        // loading the game room info
        let pre_game_room_state =
            GAME_ROOMS_STATE.load(deps.storage, params.game_room_key.clone())?;

        // specifying the win or draw and changing the balances of the contestants - the platform fee
        match params.result.clone() {
            GameRoomStatus::Started {} => return Err(error::ContractError::GameRoomNotStarted {}),
            GameRoomStatus::Win { addr } => {
                // modifying the game room state
                GAME_ROOMS_STATE.update(
                    deps.storage,
                    params.game_room_key.clone(),
                    |_| -> StdResult<_> { Ok(pre_game_room_state.get_finish_state(params.result)) },
                )?;

                // increasing the winner balance
                let tmp_fee = FEE.load(deps.storage)?;
                BALANCES.update(
                    deps.storage,
                    &Addr::unchecked(addr.clone()),
                    |balance: Option<Balance>| -> StdResult<_> {
                        Ok(balance.unwrap_or_default().unlock_and_increase(
                            cal_min_required(pre_game_room_state.prize_pool, Uint128::zero()),
                            cal_min_required(pre_game_room_state.prize_pool, tmp_fee),
                        ))
                    },
                )?;

                let loser = if pre_game_room_state.contestant1 == addr {
                    pre_game_room_state.contestant2
                } else {
                    pre_game_room_state.contestant1
                };

                // decreasing the loser balance
                BALANCES.update(
                    deps.storage,
                    &Addr::unchecked(loser),
                    |balance: Option<Balance>| -> StdResult<_> {
                        Ok(balance.unwrap_or_default().unlock_and_decrease(
                            cal_min_required(pre_game_room_state.prize_pool, Uint128::zero()),
                            cal_min_required(pre_game_room_state.prize_pool, Uint128::zero()),
                        ))
                    },
                )?;
            }
            GameRoomStatus::Draw {} => {
                // modifying the game room state
                GAME_ROOMS_STATE.update(
                    deps.storage,
                    params.game_room_key.clone(),
                    |_| -> StdResult<_> { Ok(pre_game_room_state.get_finish_state(params.result)) },
                )?;

                // increasing the winner balance
                let tmp_fee = FEE.load(deps.storage)?;
                BALANCES.update(
                    deps.storage,
                    &Addr::unchecked(pre_game_room_state.contestant1),
                    |balance: Option<Balance>| -> StdResult<_> {
                        Ok(balance.unwrap_or_default().unlock_and_decrease(
                            cal_min_required(pre_game_room_state.prize_pool, Uint128::zero()),
                            Uint128::zero(),
                        ))
                    },
                )?;

                // decreasing the loser balance
                BALANCES.update(
                    deps.storage,
                    &Addr::unchecked(pre_game_room_state.contestant2),
                    |balance: Option<Balance>| -> StdResult<_> {
                        Ok(balance.unwrap_or_default().unlock_and_decrease(
                            cal_min_required(pre_game_room_state.prize_pool, Uint128::zero()),
                            Uint128::zero(),
                        ))
                    },
                )?;
            }
        }

        // changing the game room status to finished to be able to be ongoing later

        Ok(Response::new())
    }

    pub fn collect_fees(
        deps: DepsMut,
        info: MessageInfo,
        params: CollectFeesParams,
    ) -> Result<Response, ContractError> {
        // loading the admin
        let admin_addr = ADMIN.load(deps.storage)?;

        // checking that the admin is sending the request
        if info.sender != admin_addr {
            return Err(crate::error::ContractError::Unauthorized {});
        }

        // creating the the transfer msg
        let msg = cosmwasm_std::WasmMsg::Execute {
            contract_addr: ENIGMA_DUEL_TOKEN.load(deps.storage)?.into(),
            msg: to_json_binary(&cw20::Cw20ExecuteMsg::Send {
                contract: params.receiver.clone(),
                amount: params.amount,
                msg: to_json_binary(&Withdraw {
                    user: Some(info.sender.into()),
                    amount: params.amount,
                    receiver: params.receiver.clone(),
                })?,
            })?,
            funds: vec![],
        };

        let withdraw_data = Withdraw {
            user: Some(admin_addr.into_string()),
            amount: params.amount,
            receiver: params.receiver,
        };

        Ok(Response::new()
            .add_attribute("action", "collect fees")
            .add_attribute("request_data", withdraw_data.to_string())
            .add_message(msg))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCollectedFees {} => to_json_binary(&Uint128::new(5)),
        QueryMsg::GetGameRoomState { game_room_key } => Ok(to_json_binary(
            &GAME_ROOMS_STATE
                .may_load(deps.storage, game_room_key)
                .unwrap()
                .unwrap(),
        )
        .unwrap()),
        QueryMsg::GetTotalGames {} => Ok(to_json_binary(
            &GAME_ROOMS_COUNT
                .load(deps.storage)
                .unwrap_or(Uint256::zero()),
        )
        .unwrap()),
        QueryMsg::GetUserBalance { user } => {
            let balance: Uint128 = BALANCES
                .may_load(deps.storage, &Addr::unchecked(user))
                .unwrap()
                .unwrap_or(Balance::new_zero())
                .available_balance();

            Ok(to_json_binary(&balance)?)
        }
        QueryMsg::GetUserLockedBalance { user } => {
            let balance: Uint128 = BALANCES
                .may_load(deps.storage, &Addr::unchecked(user))
                .unwrap()
                .unwrap_or(Balance::new_zero())
                .locked_balance();

            Ok(to_json_binary(&balance)?)
        }
    }
}

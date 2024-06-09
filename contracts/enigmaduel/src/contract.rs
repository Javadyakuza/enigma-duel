use std::ops::Add;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;
use execute::create_game_room;
use serde::Serialize;
use test_edt::msg;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, GameRoomStatus, GetCollectedFeesResp, GetTotalGamesResp, GetUserBalanceResp,
    InstantiateMsg, QueryMsg,
};
use crate::state::{GameRoomsState, ADMIN, BALANCES, ENIGMA_DUEL_TOKEN, FEE, GAME_ROOMS_STATE};

use self::execute::update_balance_callback;

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
        &Uint128::zero(),
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
            game_room_id: Uint128,
        } => Ok(Response::new()),
        ExecuteMsg::CollectFees { amount } => Ok(Response::new()),
        ExecuteMsg::Receive(receive_msg) => {
            execute::update_balance_callback(deps, info, receive_msg.msg)
        }
    }
}

pub mod execute {
    use cosmwasm_std::from_binary;

    use super::*;
    use crate::{
        error::{self, InsufficientBalanceErr},
        helpers::create_key_hash,
        msg::{
            GameRoomFinishParams, GameRoomIntiParams,
            UpdateBalanceMode::{self, *},
        },
    };

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
                        msg: to_binary(&cw20::Cw20ExecuteMsg::SendFrom {
                            owner: info.sender.clone().into(),
                            contract: env.contract.address.into(),
                            amount,
                            msg: to_binary(&Deposit {
                                user: Some(info.sender.into()),
                                amount,
                            })?,
                        })?,
                        funds: vec![],
                    },
                    Withdraw {
                        amount, receiver, ..
                    } => cosmwasm_std::WasmMsg::Execute {
                        contract_addr: ENIGMA_DUEL_TOKEN.load(deps.storage)?.into(),
                        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
                            contract: receiver.clone(),
                            amount,
                            msg: to_binary(&Withdraw {
                                user: Some(info.sender.into()),
                                amount,
                                receiver,
                            })?,
                        })?,
                        funds: vec![],
                    },
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
            match from_binary::<UpdateBalanceMode>(&update_mode)? {
                Deposit { amount, user } => {
                    BALANCES.update(
                        deps.storage,
                        &Addr::unchecked(user.clone().unwrap()),
                        |balance: Option<Uint128>| -> StdResult<_> {
                            Ok(balance.unwrap_or_default() + amount)
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
                        |balance: Option<Uint128>| -> StdResult<_> {
                            Ok(balance.unwrap_or_default().checked_sub(amount)?)
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
        let min_required = (params
            .prize_pool
            .checked_sub(FEE.load(deps.storage)?)
            .unwrap())
        .checked_div_euclid(Uint128::new(2))
        .unwrap();

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

        // in the following line we also check the prize pool to not be zero
        if min_required >= con_1_bal || min_required >= con_2_bal {
            return Err(error::ContractError::InsufficientBalance(
                InsufficientBalanceErr {
                    min_required,
                    current_balance: con_1_bal,
                    user: params.contestant1.clone(),
                },
            ));
        } else if min_required >= con_2_bal {
            return Err(error::ContractError::InsufficientBalance(
                InsufficientBalanceErr {
                    min_required,
                    current_balance: con_2_bal,
                    user: params.contestant2.clone(),
                },
            ));
        }

        // creating the key of the these two components as the key
        let game_room_key = create_key_hash(params.contestant1.clone(), params.contestant2.clone());
        let game_room_data = GameRoomsState {
            contestant1: params.contestant1,
            contestant2: params.contestant2,
            prize_pool: params.prize_pool,
            status: GameRoomStatus::Started {},
        };

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
                    GAME_ROOMS_STATE.save(deps.storage, game_room_key, &game_room_data)?
                }
            },
            Err(e) => return Err(error::ContractError::GameRoomLoadError { msg: e.to_string() }),
        }

        Ok(Response::new())
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

        // specifying the win or draw and changing the balances of the contestants - the platform fee

        
        // changing the game room status to finished to be able to be ongoing later

        Ok(Response::new())
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCollectedFees {} => to_json_binary(&Uint128::new(5)),
        QueryMsg::GetGameRoomState { Game_room_id: i64 } => to_json_binary(&GameRoomsState {
            contestant1: "".to_string(),
            contestant2: "".to_string(),
            prize_pool: Uint128::new(5),
            status: GameRoomStatus::Draw {},
        }),
        QueryMsg::GetTotalGames {} => to_json_binary(&Uint128::new(5)),
        QueryMsg::GetUserBalance { user } => {
            let balance: Option<Uint128> =
                BALANCES.may_load(deps.storage, &Addr::unchecked("addr0000"))?;

            Ok(to_json_binary(&balance)?)
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::msg;
//     use crate::state::Fee;

//     use super::*;
//     use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
//     use cosmwasm_std::{coin, coins, from_binary, from_json, Never};
//     use cw20::{AllowanceResponse, BalanceResponse, Cw20Coin, Expiration, MinterResponse};
//     use cw_multi_test::{App, ContractWrapper, Executor};

//     #[test]
//     fn test_callback() {
//         let mut app = App::new(|router, _, storage| {
//             router
//                 .bank
//                 .init_balance(storage, &Addr::unchecked("addr0000"), coins(50, "eth"))
//                 .unwrap()
//         });

//         // initing the enigma contract
//         let edt_code = ContractWrapper::new(
//             test_edt::contract::execute,
//             test_edt::contract::instantiate,
//             test_edt::contract::query,
//         ); // the code that is going to be saved on chain
//         let edt_code_id = app.store_code(Box::new(edt_code)); // storing the code on chain using the store code method
//         let edt_addr = app
//             .instantiate_contract(
//                 edt_code_id,
//                 Addr::unchecked("addr0000"),
//                 &test_edt::msg::InstantiateMsg {
//                     name: "test_edt".to_string(),
//                     symbol: "edt".to_string(),
//                     decimals: 9_u8,
//                     initial_balances: vec![Cw20Coin {
//                         address: "addr0000".into(),
//                         amount: Uint128::new(10_000_000_000),
//                     }],
//                     mint: Some(MinterResponse {
//                         minter: "addr0000".into(),
//                         cap: Some(Uint128::new(100_000_000_000)),
//                     }),
//                     marketing: None,
//                 },
//                 &[coin(2, "eth")],
//                 "contract",
//                 None,
//             )
//             .unwrap(); // constructing the contract

//         // initing the enigma token contract
//         let enigma_code = ContractWrapper::new(execute, instantiate, query); // the code that is going to be saved on chain
//         let enigma_code_id = app.store_code(Box::new(enigma_code)); // storing the code on chain using the store code method
//         let enigma_addr = app
//             .instantiate_contract(
//                 enigma_code_id,
//                 Addr::unchecked("addr0000"),
//                 &InstantiateMsg {
//                     fee: Fee {
//                         draw: Uint128::one(),
//                         win: Uint128::one(),
//                     },
//                     admin: "addr0000".to_string(),
//                     enigma_token_duel: edt_addr.clone().to_string(),
//                 },
//                 &[coin(2, "eth")],
//                 "enigma",
//                 Some("addr0000".to_string()),
//             )
//             .unwrap(); // constructing the contract
//                        // app.send_tokens(
//                        //     Addr::unchecked("Addr0000"),
//                        //     enigma_addr.clone(),
//                        //     &[coin(1, "eth")],
//                        // )
//                        // .unwrap();
//                        // ------------------------------------------- //
//         let all_resp = app
//             .execute_contract(
//                 Addr::unchecked("addr0000"),
//                 edt_addr.clone(),
//                 &test_edt::msg::ExecuteMsg::IncreaseAllowance {
//                     spender: enigma_addr.clone().to_string(),
//                     amount: Uint128::new(1_000_000_000),
//                     expires: None,
//                 },
//                 &[],
//             )
//             .unwrap();

//         let exe_resp = app
//             .execute_contract(
//                 Addr::unchecked("addr0000"),
//                 enigma_addr.clone(),
//                 &ExecuteMsg::IncreaseBalance {
//                     amount: Uint128::new(100_000_000),
//                     contract_addr: enigma_addr.clone(),
//                 },
//                 &[],
//             )
//             .unwrap();
//         println!("{:?}", exe_resp);
//         let query_resp: Option<Uint128> = app
//             .wrap()
//             .query_wasm_smart(
//                 &enigma_addr,
//                 &msg::QueryMsg::GetUserBalance {
//                     user: "addr0000".to_string(),
//                 },
//             )
//             .unwrap();

//         let query_resp2: BalanceResponse = app
//             .wrap()
//             .query_wasm_smart(
//                 &edt_addr.clone(),
//                 &test_edt::msg::QueryMsg::Balance {
//                     address: enigma_addr.clone().to_string(),
//                 },
//             )
//             .unwrap();
//         let query_resp3: AllowanceResponse = app
//             .wrap()
//             .query_wasm_smart(
//                 &edt_addr.clone(),
//                 &test_edt::msg::QueryMsg::Allowance {
//                     owner: "addr0000".to_string(),
//                     spender: enigma_addr.clone().to_string(),
//                 },
//             )
//             .unwrap();

//         println!("{:?}, {:?} {:?}", query_resp, query_resp2, query_resp3);
//     }

//     // testing the callback function
// }

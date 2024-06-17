#[cfg(test)]
mod tests {

    use crate::*;

    use cosmwasm_std::{coin, coins, Addr, Uint128, Uint256};
    use cw20::{BalanceResponse, Cw20Coin, MinterResponse};
    use cw_multi_test::{App, ContractWrapper, Executor};
    use msg::{
        CollectFeesParams, GameRoomFinishParams, GameRoomIntiParams, GameRoomStatus, InstantiateMsg,
    };
    use state::GameRoomsState;

    struct MockApp {
        app: App,
        edt_addr: Addr,
        enigma_addr: Addr,
    }

    pub const ENIGMA_ADMIN: &str = "addr0000";
    pub const EDT_ADMIN: &str = "addr1111";
    pub const DEPLOYER: &str = "addr2222";
    pub const USER1: &str = "addr3333";
    pub const USER2: &str = "addr4444";
    pub const USER3: &str = "addr5555";

    fn get_app() -> MockApp {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(DEPLOYER), coins(50, "eth"))
                .unwrap()
        });

        let edt_code = ContractWrapper::new(
            test_edt::contract::execute,
            test_edt::contract::instantiate,
            test_edt::contract::query,
        );
        let edt_code_id = app.store_code(Box::new(edt_code));
        let edt_addr = app
            .instantiate_contract(
                edt_code_id,
                Addr::unchecked(DEPLOYER),
                &test_edt::msg::InstantiateMsg {
                    name: "test_edt".to_string(),
                    symbol: "edt".to_string(),
                    decimals: 9_u8,
                    initial_balances: vec![
                        Cw20Coin {
                            address: USER1.into(),
                            amount: Uint128::new(10000000000),
                        },
                        Cw20Coin {
                            address: USER2.into(),
                            amount: Uint128::new(10000000000),
                        },
                    ],
                    mint: Some(MinterResponse {
                        minter: EDT_ADMIN.into(),
                        cap: Some(Uint128::new(100_000_000_000)),
                    }),
                    marketing: None,
                },
                &[coin(2, "eth")],
                "contract",
                None,
            )
            .unwrap();

        let enigma_code =
            ContractWrapper::new(contract::execute, contract::instantiate, contract::query); // the code that is going to be saved on chain
        let enigma_code_id = app.store_code(Box::new(enigma_code));
        let enigma_addr = app
            .instantiate_contract(
                enigma_code_id,
                Addr::unchecked(DEPLOYER),
                &InstantiateMsg {
                    fee: Uint128::new(100000000),
                    admin: ENIGMA_ADMIN.into(),
                    enigma_token_duel: edt_addr.clone().to_string(),
                },
                &[coin(2, "eth")],
                "enigma",
                Some(ENIGMA_ADMIN.into()),
            )
            .unwrap();

        MockApp {
            app,
            edt_addr,
            enigma_addr,
        }
    }

    fn increase_allowance(app: &mut MockApp, user: &str) {
        let _ = app
            .app
            .execute_contract(
                Addr::unchecked(user),
                app.edt_addr.clone(),
                &test_edt::msg::ExecuteMsg::IncreaseAllowance {
                    spender: app.enigma_addr.clone().to_string(),
                    amount: Uint128::new(1000000000),
                    expires: None,
                },
                &[],
            )
            .unwrap();
    }

    fn deposit(app: &mut MockApp, user: &str) {
        let _ = app
            .app
            .execute_contract(
                Addr::unchecked(user),
                app.enigma_addr.clone(),
                &crate::msg::ExecuteMsg::UpdateBalance {
                    update_mode: crate::msg::UpdateBalanceMode::Deposit {
                        user: Some(user.into()),
                        amount: Uint128::new(1000000000),
                    },
                },
                &[],
            )
            .unwrap();
    }

    fn withdraw(app: &mut MockApp, user: &str) {
        app.app
            .execute_contract(
                Addr::unchecked(USER1),
                app.enigma_addr.clone(),
                &crate::msg::ExecuteMsg::UpdateBalance {
                    update_mode: crate::msg::UpdateBalanceMode::Withdraw {
                        user: Some(user.into()),
                        amount: Uint128::new(1000000000),
                        receiver: user.into(),
                    },
                },
                &[],
            )
            .unwrap();
    }

    fn create_gr(app: &mut MockApp) -> String {
        match app.app.execute_contract(
            Addr::unchecked(ENIGMA_ADMIN),
            app.enigma_addr.clone(),
            &crate::msg::ExecuteMsg::CreateGameRoom {
                game_room_init_params: GameRoomIntiParams {
                    contestant1: USER1.into(),
                    contestant2: USER2.into(),
                    prize_pool: Uint128::new(1500000000),
                    status: msg::GameRoomStatus::Started {},
                },
            },
            &[],
        ) {
            Ok(res) => {
                println!("{:?}", res.events[1].attributes[2].value);
                res.events[1].attributes[2].value.clone()
            }
            Err(err) => {
                println!("error: {}", err);
                err.to_string()
            }
        }
    }

    fn finish_gr(app: &mut MockApp, game_room_key: String, result: GameRoomStatus) {
        app.app
            .execute_contract(
                Addr::unchecked(ENIGMA_ADMIN),
                app.enigma_addr.clone(),
                &crate::msg::ExecuteMsg::FinishGameRoom {
                    game_room_finish_params: GameRoomFinishParams {
                        game_room_key,
                        result,
                    },
                },
                &[],
            )
            .unwrap();
    }

    fn collect_fees(app: &mut MockApp, receiver: String, amount: Uint128) {
        app.app
            .execute_contract(
                Addr::unchecked(ENIGMA_ADMIN),
                app.enigma_addr.clone(),
                &crate::msg::ExecuteMsg::CollectFees {
                    collect_fees_params: CollectFeesParams { amount, receiver },
                },
                &[],
            )
            .unwrap();
    }

    #[test]
    fn test_deposit() {
        let mut app = get_app();

        increase_allowance(&mut app, USER1);
        deposit(&mut app, USER1);

        let enigma_balance: Option<Uint128> = app
            .app
            .wrap()
            .query_wasm_smart(
                app.enigma_addr.clone(),
                &msg::QueryMsg::GetUserBalance { user: USER1.into() },
            )
            .unwrap();

        let edt_balance: BalanceResponse = app
            .app
            .wrap()
            .query_wasm_smart(
                app.edt_addr,
                &test_edt::msg::QueryMsg::Balance {
                    address: app.enigma_addr.clone().into(),
                },
            )
            .unwrap();

        assert_eq!(enigma_balance.unwrap(), Uint128::new(1000000000));
        println!("{}", edt_balance.balance);
        assert_eq!(edt_balance.balance, Uint128::new(1000000000));
    }

    #[test]
    fn test_withdraw() {
        let mut app = get_app();

        increase_allowance(&mut app, USER1);
        deposit(&mut app, USER1);
        withdraw(&mut app, USER1);

        let enigma_balance: Option<Uint128> = app
            .app
            .wrap()
            .query_wasm_smart(
                app.enigma_addr,
                &msg::QueryMsg::GetUserBalance { user: USER1.into() },
            )
            .unwrap();

        let edt_balance: Option<BalanceResponse> = app
            .app
            .wrap()
            .query_wasm_smart(
                app.edt_addr,
                &test_edt::msg::QueryMsg::Balance {
                    address: USER1.into(),
                },
            )
            .unwrap();

        assert_eq!(enigma_balance.unwrap(), Uint128::new(0));
        assert_eq!(edt_balance.unwrap().balance, Uint128::new(10000000000));
    }

    #[test]
    fn test_create_game_room() {
        let mut app = get_app();

        increase_allowance(&mut app, USER1);
        deposit(&mut app, USER1);
        increase_allowance(&mut app, USER2);
        deposit(&mut app, USER2);

        let game_room_key = create_gr(&mut app);

        let gr_state: Option<GameRoomsState> = app
            .app
            .wrap()
            .query_wasm_smart(
                app.enigma_addr.clone(),
                &msg::QueryMsg::GetGameRoomState { game_room_key },
            )
            .unwrap();
        let gr_count: Option<Uint256> = app
            .app
            .wrap()
            .query_wasm_smart(app.enigma_addr.clone(), &msg::QueryMsg::GetTotalGames {})
            .unwrap();
        // checking if the balance locks are updated

        let con_1_bal: Option<Uint128> = app
            .app
            .wrap()
            .query_wasm_smart(
                app.enigma_addr.clone(),
                &msg::QueryMsg::GetUserBalance { user: USER1.into() },
            )
            .unwrap();
        let con_2_bal: Option<Uint128> = app
            .app
            .wrap()
            .query_wasm_smart(
                app.enigma_addr,
                &msg::QueryMsg::GetUserBalance { user: USER2.into() },
            )
            .unwrap();
        assert_eq!(con_1_bal.unwrap(), Uint128::new(250000000));
        assert_eq!(con_2_bal.unwrap(), Uint128::new(250000000));
        assert_eq!(gr_state.unwrap().status, GameRoomStatus::Started {});
        assert_eq!(gr_count.unwrap(), Uint256::one());
    }

    #[test]
    fn test_finish_game_room_win() {
        let mut app = get_app();

        increase_allowance(&mut app, USER1);
        deposit(&mut app, USER1);
        increase_allowance(&mut app, USER2);
        deposit(&mut app, USER2);

        let game_room_key = create_gr(&mut app);

        let _ = finish_gr(
            &mut app,
            game_room_key.clone(),
            GameRoomStatus::Win { addr: USER1.into() },
        );
        let con_1_bal: Option<Uint128> = app
            .app
            .wrap()
            .query_wasm_smart(
                app.enigma_addr.clone(),
                &msg::QueryMsg::GetUserBalance { user: USER1.into() },
            )
            .unwrap();
        let con_2_bal: Option<Uint128> = app
            .app
            .wrap()
            .query_wasm_smart(
                app.enigma_addr.clone(),
                &msg::QueryMsg::GetUserBalance { user: USER2.into() },
            )
            .unwrap();

        let gr_state: Option<GameRoomsState> = app
            .app
            .wrap()
            .query_wasm_smart(
                app.enigma_addr.clone(),
                &msg::QueryMsg::GetGameRoomState { game_room_key },
            )
            .unwrap();

        let collected_fees: Option<Uint128> = app
            .app
            .wrap()
            .query_wasm_smart(app.enigma_addr.clone(), &msg::QueryMsg::GetCollectedFees {})
            .unwrap();
        // the user one wins so the the balance must be the prize pool - fee + old balance => 1_500_000_000 - 1_00_000_000 + 1_000_000_000 = 2_400_000_000
        assert_eq!(con_1_bal.unwrap(), Uint128::new(2400000000));
        // the user two lost the game so the balance must be => old balance - prize pool / 2  = 1_000_000_00 - 750_000_000 = 250_000_000
        assert_eq!(con_2_bal.unwrap(), Uint128::new(250000000));
        assert_eq!(collected_fees.unwrap(), Uint128::new(2_00_000_000));

        assert_eq!(
            gr_state.unwrap().status,
            GameRoomStatus::Win { addr: USER1.into() }
        );
    }

    #[test]
    fn test_finish_game_room_draw() {
        let mut app = get_app();

        increase_allowance(&mut app, USER1);
        deposit(&mut app, USER1);
        increase_allowance(&mut app, USER2);
        deposit(&mut app, USER2);

        let game_room_key = create_gr(&mut app);

        let _ = finish_gr(&mut app, game_room_key.clone(), GameRoomStatus::Draw {});
        let con_1_bal: Option<Uint128> = app
            .app
            .wrap()
            .query_wasm_smart(
                app.enigma_addr.clone(),
                &msg::QueryMsg::GetUserBalance { user: USER1.into() },
            )
            .unwrap();
        let con_2_bal: Option<Uint128> = app
            .app
            .wrap()
            .query_wasm_smart(
                app.enigma_addr.clone(),
                &msg::QueryMsg::GetUserBalance { user: USER2.into() },
            )
            .unwrap();

        let gr_state: Option<GameRoomsState> = app
            .app
            .wrap()
            .query_wasm_smart(
                app.enigma_addr.clone(),
                &msg::QueryMsg::GetGameRoomState { game_room_key },
            )
            .unwrap();
        assert_eq!(con_1_bal.unwrap(), Uint128::new(1000000000));
        assert_eq!(con_2_bal.unwrap(), Uint128::new(1000000000));

        assert_eq!(gr_state.unwrap().status, GameRoomStatus::Draw {});
    }

    #[test]
    fn test_collect_fees() {
        let mut app = get_app();

        increase_allowance(&mut app, USER1);
        deposit(&mut app, USER1);
        increase_allowance(&mut app, USER2);
        deposit(&mut app, USER2);

        let game_room_key = create_gr(&mut app);

        let _ = finish_gr(
            &mut app,
            game_room_key.clone(),
            GameRoomStatus::Win { addr: USER1.into() },
        );
        let admin_bal: Option<Uint128> = app
            .app
            .wrap()
            .query_wasm_smart(
                app.enigma_addr.clone(),
                &msg::QueryMsg::GetUserBalance {
                    user: ENIGMA_ADMIN.into(),
                },
            )
            .unwrap();

        assert_eq!(Uint128::new(2_00_000_000), admin_bal.unwrap());

        collect_fees(&mut app, USER3.into(), admin_bal.unwrap());

        let edt_balance: BalanceResponse = app
            .app
            .wrap()
            .query_wasm_smart(
                app.edt_addr,
                &test_edt::msg::QueryMsg::Balance {
                    address: USER3.into(),
                },
            )
            .unwrap();
        assert_eq!(Uint128::new(2_00_000_000), edt_balance.balance);

        let admin_bal: Option<Uint128> = app
            .app
            .wrap()
            .query_wasm_smart(
                app.enigma_addr.clone(),
                &msg::QueryMsg::GetUserBalance {
                    user: ENIGMA_ADMIN.into(),
                },
            )
            .unwrap();

        assert_eq!(Uint128::zero(), admin_bal.unwrap());
    }
}

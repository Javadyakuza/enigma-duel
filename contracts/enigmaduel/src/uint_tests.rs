#[cfg(test)]
mod tests {
    use crate::*;

    use cosmwasm_std::{coin, coins, Addr, Uint128};
    use cw20::{Balance, BalanceResponse, Cw20Coin, MinterResponse};
    use cw_multi_test::{App, ContractWrapper, Executor};
    use msg::InstantiateMsg;

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
                    fee: Uint128::one(),
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

    fn increase_allowance(app: &mut MockApp) {
        let _ = app
            .app
            .execute_contract(
                Addr::unchecked(USER1),
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
    fn deposit(app: &mut MockApp) {
        let _ = app
            .app
            .execute_contract(
                Addr::unchecked(USER1),
                app.enigma_addr.clone(),
                &crate::msg::ExecuteMsg::UpdateBalance {
                    update_mode: crate::msg::UpdateBalanceMode::Deposit {
                        user: Some(USER1.into()),
                        amount: Uint128::new(1000000000),
                    },
                },
                &[],
            )
            .unwrap();
    }
    fn withdraw(app: &mut MockApp) {
        match app.app.execute_contract(
            Addr::unchecked(USER1),
            app.enigma_addr.clone(),
            &crate::msg::ExecuteMsg::UpdateBalance {
                update_mode: crate::msg::UpdateBalanceMode::Withdraw {
                    user: Some(USER1.into()),
                    amount: Uint128::new(1000000000),
                    receiver: USER1.into(),
                },
            },
            &[],
        ) {
            Ok(res) => {}
            Err(err) => {
                println!("error: {}", err);
            }
        }
    }

    #[test]
    fn test_deposit() {
        let mut app = get_app();

        increase_allowance(&mut app);
        deposit(&mut app);

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

        increase_allowance(&mut app);
        deposit(&mut app);
        withdraw(&mut app);

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
    // testing the callback function
}

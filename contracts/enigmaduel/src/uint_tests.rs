#[cfg(test)]
mod tests {
    use crate::*;

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, from_binary, from_json, Addr, Never, Uint128};
    use cw20::{AllowanceResponse, BalanceResponse, Cw20Coin, Expiration, MinterResponse};
    use cw_multi_test::{App, ContractWrapper, Executor};
    use msg::InstantiateMsg;

    struct MockApp {
        app: App,
        edt_addr: Addr,
        enigma_addr: Addr,
    }

    pub const ENIGMA_ADMIN: &str = "enigmaAdmin";
    pub const EDT_ADMIN: &str = "edtAdmin";
    pub const DEPLOYER: &str = "deployer";
    pub const USER1: &str = "user1";
    pub const USER2: &str = "user2";

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
                            amount: Uint128::new(10_000_000_000),
                        },
                        Cw20Coin {
                            address: USER2.into(),
                            amount: Uint128::new(10_000_000_000),
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

=        let enigma_code =
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

    #[test]
    fn deposit() {
        let mut app = get_app();

        let all_resp = app
            .app
            .execute_contract(
                Addr::unchecked("addr0000"),
                app.edt_addr.clone(),
                &test_edt::msg::ExecuteMsg::IncreaseAllowance {
                    spender: app.enigma_addr.clone().to_string(),
                    amount: Uint128::new(1_000_000_000),
                    expires: None,
                },
                &[],
            )
            .unwrap();

        let exe_resp = app
            .app
            .execute_contract(
                Addr::unchecked("addr0000"),
                app.enigma_addr.clone(),
                &crate::msg::ExecuteMsg::UpdateBalance {
                    update_mode: crate::msg::UpdateBalanceMode::Deposit {
                        user: Some("addr0000".to_string()),
                        amount: Uint128::new(100_000_000),
                    },
                },
                &[],
            )
            .unwrap();
        println!("{:?}", exe_resp);
        let balance: Option<Uint128> = app
            .app
            .wrap()
            .query_wasm_smart(
                &app.enigma_addr,
                &msg::QueryMsg::GetUserBalance {
                    user: "addr0000".to_string(),
                },
            )
            .unwrap();
    }

    // testing the callback function
}

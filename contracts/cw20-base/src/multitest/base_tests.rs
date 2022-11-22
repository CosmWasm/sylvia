use cosmwasm_std::{Addr, Binary, StdError, Uint128};
use cw20_allowances::responses::{AllAllowancesResponse, SpenderAllowanceInfo};
use cw_multi_test::App;
use cw_utils::Expiration;

use crate::contract::InstantiateMsgData;
use crate::error::ContractError;
use crate::multitest::proxy::Cw20BaseCodeId;
use crate::multitest::receiver_contract::ReceiverContractCodeId;
use crate::responses::{BalanceResponse, Cw20Coin, TokenInfoResponse};

#[test]
fn basic() {
    let mut app = App::default();

    let amount = Uint128::from(11223344u128);
    let owner = Addr::unchecked("addr0001");

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 9,
                initial_balances: vec![Cw20Coin {
                    address: owner.clone().into(),
                    amount,
                }],
                mint: None,
                marketing: None,
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    let resp = contract.token_info(&app).unwrap();

    assert_eq!(
        resp,
        TokenInfoResponse {
            name: "Cash Token".to_string(),
            symbol: "CASH".to_string(),
            decimals: 9,
            total_supply: amount,
        }
    );

    let resp = contract.balance(&app, owner.to_string()).unwrap();

    assert_eq!(resp, BalanceResponse { balance: amount });
}

#[test]
fn instantiate_multiple_accounts() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0000");
    let amount1 = Uint128::from(11223344u128);
    let addr1 = Addr::unchecked("addr0001");
    let amount2 = Uint128::from(7890987u128);
    let addr2 = Addr::unchecked("addr0002");

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    // Fails with duplicate addresses
    let err = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 6,
                initial_balances: vec![
                    Cw20Coin {
                        address: addr1.clone().into(),
                        amount: amount1,
                    },
                    Cw20Coin {
                        address: addr1.clone().into(),
                        amount: amount2,
                    },
                ],
                mint: None,
                marketing: None,
            },
            "Cw20 contract",
            None,
        )
        .unwrap_err();

    assert_eq!(err, ContractError::DuplicateInitialBalanceAddresses {});

    // Works with unique addresses
    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Bash Token".to_string(),
                symbol: "BASH".to_string(),
                decimals: 6,
                initial_balances: vec![
                    Cw20Coin {
                        address: addr1.clone().into(),
                        amount: amount1,
                    },
                    Cw20Coin {
                        address: addr2.clone().into(),
                        amount: amount2,
                    },
                ],
                mint: None,
                marketing: None,
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    let resp = contract.token_info(&app).unwrap();
    assert_eq!(
        resp,
        TokenInfoResponse {
            name: "Bash Token".to_string(),
            symbol: "BASH".to_string(),
            decimals: 6,
            total_supply: amount1 + amount2,
        }
    );
    let resp = contract.balance(&app, addr1.to_string()).unwrap();
    assert_eq!(resp.balance, amount1);
    let resp = contract.balance(&app, addr2.to_string()).unwrap();
    assert_eq!(resp.balance, amount2);
}

#[test]
fn queries_work() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0000");
    let amount = Uint128::from(12340000u128);
    let addr = Addr::unchecked("addr0001");

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 3,
                initial_balances: vec![Cw20Coin {
                    address: owner.clone().into(),
                    amount,
                }],
                mint: None,
                marketing: None,
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    let resp = contract.token_info(&app).unwrap();

    // Check meta query
    assert_eq!(
        resp,
        TokenInfoResponse {
            name: "Cash Token".to_string(),
            symbol: "CASH".to_string(),
            decimals: 3,
            total_supply: amount
        }
    );

    // Check owner balance
    let resp = contract.balance(&app, owner.to_string()).unwrap();

    assert_eq!(resp.balance, amount);

    // Check addr balance (Empty)
    let resp = contract.balance(&app, addr.to_string()).unwrap();

    assert_eq!(resp.balance, Uint128::zero());
}

#[test]
fn transfer() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0000");
    let addr = Addr::unchecked("addr0001");
    let amount = Uint128::from(12340000u128);
    let transfer = Uint128::from(76543u128);
    let too_much = Uint128::from(12340321u128);

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 3,
                initial_balances: vec![Cw20Coin {
                    address: owner.clone().into(),
                    amount,
                }],
                mint: None,
                marketing: None,
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    // cannot transfer nothing
    let err = contract
        .transfer(&mut app, &owner, addr.to_string(), Uint128::zero())
        .unwrap_err();
    assert_eq!(err, ContractError::InvalidZeroAmount {});

    // cannot send more than we have
    let err = contract
        .transfer(&mut app, &owner, addr.to_string(), too_much)
        .unwrap_err();

    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // cannot send from empty account
    let err = contract
        .transfer(&mut app, &addr, owner.to_string(), transfer)
        .unwrap_err();

    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // valid transfer
    contract
        .transfer(&mut app, &owner, addr.to_string(), transfer)
        .unwrap();

    // Check balance
    let remainder = amount.checked_sub(transfer).unwrap();

    let resp = contract.balance(&app, addr.to_string()).unwrap();
    assert_eq!(resp.balance, transfer);
    let resp = contract.balance(&app, owner.to_string()).unwrap();
    assert_eq!(resp.balance, remainder);
    let resp = contract.token_info(&app).unwrap();
    assert_eq!(resp.total_supply, amount);
}

#[test]
fn burn() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0000");
    let addr = Addr::unchecked("addr0001");
    let amount = Uint128::from(12340000u128);
    let burn = Uint128::from(76543u128);
    let too_much = Uint128::from(12340321u128);

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 3,
                initial_balances: vec![Cw20Coin {
                    address: owner.clone().into(),
                    amount,
                }],
                mint: None,
                marketing: None,
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    // cannot burn nothing
    let err = contract
        .burn(&mut app, &owner, Uint128::zero())
        .unwrap_err();
    assert_eq!(err, ContractError::InvalidZeroAmount {});

    let resp = contract.token_info(&app).unwrap();
    assert_eq!(resp.total_supply, amount);

    // cannot burn more than we have
    let err = contract.burn(&mut app, &owner, too_much).unwrap_err();

    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));
    let resp = contract.token_info(&app).unwrap();
    assert_eq!(resp.total_supply, amount);

    // cannot send from empty account
    let err = contract.burn(&mut app, &addr, burn).unwrap_err();

    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // valid burn reduces total supply
    contract.burn(&mut app, &owner, burn).unwrap();

    // check balance
    let remainder = amount.checked_sub(burn).unwrap();
    let resp = contract.balance(&app, owner.to_string()).unwrap();
    assert_eq!(resp.balance, remainder);
    let resp = contract.token_info(&app).unwrap();
    assert_eq!(resp.total_supply, remainder);
}

#[test]
fn send() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0000");
    let amount = Uint128::from(12340000u128);
    let too_much = Uint128::from(12340321u128);
    let transfer = Uint128::from(76543u128);
    let send_msg = Binary::from(r#"{"some":123}"#.as_bytes());

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    // Receiver contract
    let receiver_code_id = ReceiverContractCodeId::store_code(&mut app);
    let receiver_contract = receiver_code_id
        .instantiate(&mut app, &owner, "cool-dex")
        .unwrap();

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 3,
                initial_balances: vec![Cw20Coin {
                    address: owner.clone().into(),
                    amount,
                }],
                mint: None,
                marketing: None,
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    let err = contract
        .send(
            &mut app,
            &owner,
            receiver_contract.addr().to_string(),
            Uint128::zero(),
            send_msg.clone(),
        )
        .unwrap_err();

    assert_eq!(err, ContractError::InvalidZeroAmount {});

    // cannot send more than we have
    let err = contract
        .send(
            &mut app,
            &owner,
            receiver_contract.addr().to_string(),
            too_much,
            send_msg.clone(),
        )
        .unwrap_err();
    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // valid transfer
    contract
        .send(
            &mut app,
            &owner,
            receiver_contract.addr().to_string(),
            transfer,
            send_msg,
        )
        .unwrap();

    // ensure balance is properly transferred
    let remainder = amount.checked_sub(transfer).unwrap();
    let resp = contract.balance(&app, owner.to_string()).unwrap();
    assert_eq!(resp.balance, remainder);
    let resp = contract
        .balance(&app, receiver_contract.addr().to_string())
        .unwrap();
    assert_eq!(resp.balance, transfer);
    let resp = contract.token_info(&app).unwrap();
    assert_eq!(resp.total_supply, amount);
}

#[test]
fn migrate() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0000");
    let spender = Addr::unchecked("addr0001");
    let code_id = Cw20BaseCodeId::store_code(&mut app);
    let amount = Uint128::new(100);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Token".to_string(),
                symbol: "TOKEN".to_string(),
                decimals: 6,
                initial_balances: vec![Cw20Coin {
                    address: owner.clone().into(),
                    amount,
                }],
                mint: None,
                marketing: None,
            },
            "Cw20 contract",
            Some(owner.to_string()),
        )
        .unwrap();

    // no allowance to start
    let resp = contract
        .all_allowances(&app, owner.to_string(), None, None)
        .unwrap();
    assert_eq!(resp, AllAllowancesResponse::default());

    // Set allowance
    let allow1 = Uint128::new(7777);
    let expires = Expiration::AtHeight(123_456);
    contract
        .increase_allowance(&mut app, &owner, spender.to_string(), allow1, Some(expires))
        .unwrap();

    // Now migrate
    contract
        .migrate(&mut app, &owner, code_id.code_id())
        .unwrap();

    // Smoke check that the contract still works.
    let resp = contract.balance(&app, owner.to_string()).unwrap();
    assert_eq!(resp.balance, Uint128::new(100));

    // Confirm that the allowance per spender is there
    let resp = contract
        .all_spender_allowances(&app, spender.to_string(), None, None)
        .unwrap();
    assert_eq!(
        resp.allowances,
        &[SpenderAllowanceInfo {
            owner: owner.to_string(),
            allowance: allow1,
            expires
        }]
    );
}

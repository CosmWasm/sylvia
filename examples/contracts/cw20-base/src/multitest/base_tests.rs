use cw20_allowances::responses::{AllAllowancesResponse, SpenderAllowanceInfo};
use cw_utils::Expiration;
use sylvia::cw_multi_test::IntoBech32;
use sylvia::cw_std::{Binary, StdError, Uint128};
use sylvia::multitest::App;

use crate::contract::sv::mt::{CodeId, Cw20BaseProxy};
use crate::contract::InstantiateMsgData;
use crate::error::ContractError;
use crate::multitest::receiver_contract::sv::mt::CodeId as ReceiverCodeId;
use crate::responses::{BalanceResponse, Cw20Coin, TokenInfoResponse};
use cw20_allowances::sv::mt::Cw20AllowancesProxy;

#[test]
fn basic() {
    let app = App::default();

    let amount = Uint128::from(11223344u128);
    let owner = "owner".into_bech32();

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Cash Token".to_string(),
            symbol: "CASH".to_string(),
            decimals: 9,
            initial_balances: vec![Cw20Coin {
                address: owner.to_string(),
                amount,
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(&owner)
        .unwrap();

    let resp = contract.token_info().unwrap();

    assert_eq!(
        resp,
        TokenInfoResponse {
            name: "Cash Token".to_string(),
            symbol: "CASH".to_string(),
            decimals: 9,
            total_supply: amount,
        }
    );

    let resp = contract.balance(owner.to_string()).unwrap();

    assert_eq!(resp, BalanceResponse { balance: amount });
}

#[test]
fn instantiate_multiple_accounts() {
    let app = App::default();

    let owner = "addr0000".into_bech32();
    let amount1 = Uint128::from(11223344u128);
    let addr1 = "addr0001".into_bech32();
    let amount2 = Uint128::from(7890987u128);
    let addr2 = "addr0002".into_bech32();

    let code_id = CodeId::store_code(&app);

    // Fails with duplicate addresses
    let err = code_id
        .instantiate(InstantiateMsgData {
            name: "Cash Token".to_string(),
            symbol: "CASH".to_string(),
            decimals: 6,
            initial_balances: vec![
                Cw20Coin {
                    address: addr1.to_string(),
                    amount: amount1,
                },
                Cw20Coin {
                    address: addr1.to_string(),
                    amount: amount2,
                },
            ],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(&owner)
        .unwrap_err();

    assert_eq!(err, ContractError::DuplicateInitialBalanceAddresses);

    // Works with unique addresses
    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Bash Token".to_string(),
            symbol: "BASH".to_string(),
            decimals: 6,
            initial_balances: vec![
                Cw20Coin {
                    address: addr1.to_string(),
                    amount: amount1,
                },
                Cw20Coin {
                    address: addr2.to_string(),
                    amount: amount2,
                },
            ],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(&owner)
        .unwrap();

    let resp = contract.token_info().unwrap();
    assert_eq!(
        resp,
        TokenInfoResponse {
            name: "Bash Token".to_string(),
            symbol: "BASH".to_string(),
            decimals: 6,
            total_supply: amount1 + amount2,
        }
    );
    let resp = contract.balance(addr1.to_string()).unwrap();
    assert_eq!(resp.balance, amount1);
    let resp = contract.balance(addr2.to_string()).unwrap();
    assert_eq!(resp.balance, amount2);
}

#[test]
fn queries_work() {
    let app = App::default();

    let owner = "addr0000".into_bech32();
    let amount = Uint128::from(12340000u128);
    let addr = "addr0001".into_bech32();

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Cash Token".to_string(),
            symbol: "CASH".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: owner.to_string(),
                amount,
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(&owner)
        .unwrap();

    let resp = contract.token_info().unwrap();

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
    let resp = contract.balance(owner.to_string()).unwrap();

    assert_eq!(resp.balance, amount);

    // Check addr balance (Empty)
    let resp = contract.balance(addr.to_string()).unwrap();

    assert_eq!(resp.balance, Uint128::zero());
}

#[test]
fn transfer() {
    let app = App::default();

    let owner = "addr0000".into_bech32();
    let addr = "addr0001".into_bech32();
    let amount = Uint128::from(12340000u128);
    let transfer = Uint128::from(76543u128);
    let too_much = Uint128::from(12340321u128);

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Cash Token".to_string(),
            symbol: "CASH".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: owner.to_string(),
                amount,
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(&owner)
        .unwrap();

    // cannot transfer nothing
    let err = contract
        .transfer(addr.to_string(), Uint128::zero())
        .call(&owner)
        .unwrap_err();
    assert_eq!(err, ContractError::InvalidZeroAmount);

    // cannot send more than we have
    let err = contract
        .transfer(addr.to_string(), too_much)
        .call(&owner)
        .unwrap_err();

    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // cannot send from empty account
    let err = contract
        .transfer(owner.to_string(), transfer)
        .call(&addr)
        .unwrap_err();

    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // valid transfer
    contract
        .transfer(addr.to_string(), transfer)
        .call(&owner)
        .unwrap();

    // Check balance
    let remainder = amount.checked_sub(transfer).unwrap();

    let resp = contract.balance(addr.to_string()).unwrap();
    assert_eq!(resp.balance, transfer);
    let resp = contract.balance(owner.to_string()).unwrap();
    assert_eq!(resp.balance, remainder);
    let resp = contract.token_info().unwrap();
    assert_eq!(resp.total_supply, amount);
}

#[test]
fn burn() {
    let app = App::default();

    let owner = "addr0000".into_bech32();
    let addr = "addr0001".into_bech32();
    let amount = Uint128::from(12340000u128);
    let burn = Uint128::from(76543u128);
    let too_much = Uint128::from(12340321u128);

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Cash Token".to_string(),
            symbol: "CASH".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: owner.to_string(),
                amount,
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(&owner)
        .unwrap();

    // cannot burn nothing
    let err = contract.burn(Uint128::zero()).call(&owner).unwrap_err();
    assert_eq!(err, ContractError::InvalidZeroAmount);

    let resp = contract.token_info().unwrap();
    assert_eq!(resp.total_supply, amount);

    // cannot burn more than we have
    let err = contract.burn(too_much).call(&owner).unwrap_err();

    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));
    let resp = contract.token_info().unwrap();
    assert_eq!(resp.total_supply, amount);

    // cannot send from empty account
    let err = contract.burn(burn).call(&addr).unwrap_err();

    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // valid burn reduces total supply
    contract.burn(burn).call(&owner).unwrap();

    // check balance
    let remainder = amount.checked_sub(burn).unwrap();
    let resp = contract.balance(owner.to_string()).unwrap();
    assert_eq!(resp.balance, remainder);
    let resp = contract.token_info().unwrap();
    assert_eq!(resp.total_supply, remainder);
}

#[test]
fn send() {
    let app = App::default();

    let owner = "addr0000".into_bech32();
    let amount = Uint128::from(12340000u128);
    let too_much = Uint128::from(12340321u128);
    let transfer = Uint128::from(76543u128);
    let send_msg = Binary::from(r#"{"some":123}"#.as_bytes());

    let code_id = CodeId::store_code(&app);

    // Receiver contract
    let receiver_code_id = ReceiverCodeId::store_code(&app);
    let receiver_contract = receiver_code_id
        .instantiate()
        .with_label("cool-dex")
        .call(&owner)
        .unwrap();

    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Cash Token".to_string(),
            symbol: "CASH".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: owner.to_string(),
                amount,
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(&owner)
        .unwrap();

    let err = contract
        .send(
            receiver_contract.contract_addr.to_string(),
            Uint128::zero(),
            send_msg.clone(),
        )
        .call(&owner)
        .unwrap_err();

    assert_eq!(err, ContractError::InvalidZeroAmount);

    // cannot send more than we have
    let err = contract
        .send(
            receiver_contract.contract_addr.to_string(),
            too_much,
            send_msg.clone(),
        )
        .call(&owner)
        .unwrap_err();
    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // valid transfer
    contract
        .send(
            receiver_contract.contract_addr.to_string(),
            transfer,
            send_msg,
        )
        .call(&owner)
        .unwrap();

    // ensure balance is properly transferred
    let remainder = amount.checked_sub(transfer).unwrap();
    let resp = contract.balance(owner.to_string()).unwrap();
    assert_eq!(resp.balance, remainder);
    let resp = contract
        .balance(receiver_contract.contract_addr.to_string())
        .unwrap();
    assert_eq!(resp.balance, transfer);
    let resp = contract.token_info().unwrap();
    assert_eq!(resp.total_supply, amount);
}

#[test]
fn migrate() {
    let app = App::default();

    let owner = "addr0000".into_bech32();
    let spender = "addr0001".into_bech32();
    let code_id = CodeId::store_code(&app);
    let amount = Uint128::new(100);

    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Token".to_string(),
            symbol: "TOKEN".to_string(),
            decimals: 6,
            initial_balances: vec![Cw20Coin {
                address: owner.to_string(),
                amount,
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .with_admin(owner.as_str())
        .call(&owner)
        .unwrap();

    // no allowance to start
    let resp = contract
        .all_allowances(owner.to_string(), None, None)
        .unwrap();
    assert_eq!(resp, AllAllowancesResponse::default());

    // Set allowance
    let allow1 = Uint128::new(7777);
    let expires = Expiration::AtHeight(123_456);
    contract
        .increase_allowance(spender.to_string(), allow1, Some(expires))
        .call(&owner)
        .unwrap();

    // Now migrate
    contract.migrate().call(&owner, code_id.code_id()).unwrap();

    // Smoke check that the contract still works.
    let resp = contract.balance(owner.to_string()).unwrap();
    assert_eq!(resp.balance, Uint128::new(100));

    // Confirm that the allowance per spender is there
    let resp = contract
        .all_spender_allowances(spender.to_string(), None, None)
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

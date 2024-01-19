use cosmwasm_std::{StdError, Uint128};
use cw20_minting::responses::MinterResponse;
use sylvia::multitest::App;

use crate::contract::sv::multitest_utils::CodeId;
use crate::contract::InstantiateMsgData;
use crate::error::ContractError;
use crate::minting::sv::test_utils::Cw20Minting;
use crate::responses::{Cw20Coin, TokenInfoResponse};

#[test]
fn mintable() {
    let app = App::default();

    let owner = "addr0001";
    let minter = "addr0002";
    let amount = Uint128::new(11223344);
    let limit = Uint128::new(511223344);

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
            mint: Some(MinterResponse {
                minter: minter.to_string(),
                cap: Some(limit),
            }),
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    // read token info
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

    // get owner balance
    let resp = contract.balance(owner.to_string()).unwrap();

    assert_eq!(resp.balance, Uint128::new(11223344));

    // get minter balance
    let resp = contract.balance(minter.to_string()).unwrap();

    assert_eq!(resp.balance, Uint128::new(0));

    // get minter
    let resp = contract.minter().unwrap();

    assert_eq!(
        resp,
        Some(MinterResponse {
            minter: minter.to_string(),
            cap: Some(limit),
        })
    );
}

#[test]
fn mintable_over_cap() {
    let app = App::default();

    let owner = "addr0001";
    let minter = "addr0002";
    let amount = Uint128::new(11223344);
    let limit = Uint128::new(11223300);

    let code_id = CodeId::store_code(&app);

    let err = code_id
        .instantiate(InstantiateMsgData {
            name: "Cash Token".to_string(),
            symbol: "CASH".to_string(),
            decimals: 9,
            initial_balances: vec![Cw20Coin {
                address: owner.to_string(),
                amount,
            }],
            mint: Some(MinterResponse {
                minter: minter.to_string(),
                cap: Some(limit),
            }),
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap_err();

    assert_eq!(
        err,
        StdError::generic_err("Initial supply greater than cap").into()
    );
}

#[test]
fn can_mint_by_minter() {
    let app = App::default();

    let owner = "addr0001";
    let minter = "addr0002";
    let winner = "lucky";
    let prize = Uint128::new(222_222_222);
    let limit = Uint128::new(511223344);
    let amount = Uint128::new(11223344);

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
            mint: Some(MinterResponse {
                minter: minter.to_string(),
                cap: Some(limit),
            }),
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    // minter can mint coins to some winner
    contract
        .mint(winner.to_string(), prize)
        .call(minter)
        .unwrap();

    // but cannot mint nothing
    let err = contract
        .mint(winner.to_string(), Uint128::zero())
        .call(minter)
        .unwrap_err();

    assert_eq!(err, ContractError::InvalidZeroAmount);

    // but if it exceeds cap (even over multiple rounds), it fails
    // cap is enforced
    let err = contract
        .mint(winner.to_string(), Uint128::new(333_222_222))
        .call(minter)
        .unwrap_err();

    assert_eq!(err, ContractError::CannotExceedCap);
}

#[test]
fn others_cannot_mint() {
    let app = App::default();

    let owner = "addr0001";
    let minter = "addr0002";
    let winner = "lucky";
    let prize = Uint128::new(222_222_222);
    let limit = Uint128::new(511223344);
    let amount = Uint128::new(1234);

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
            mint: Some(MinterResponse {
                minter: minter.to_string(),
                cap: Some(limit),
            }),
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    // minter can mint coins to some winner
    contract
        .mint(winner.to_string(), prize)
        .call(minter)
        .unwrap();

    // but cannot mint nothing
    let err = contract
        .mint(winner.to_string(), Uint128::zero())
        .call(minter)
        .unwrap_err();

    assert_eq!(err, ContractError::InvalidZeroAmount);

    // but if it exceeds cap (even over multiple rounds), it fails
    // cap is enforced
    let err = contract
        .mint(winner.to_string(), Uint128::new(333_222_222))
        .call(minter)
        .unwrap_err();

    assert_eq!(err, ContractError::CannotExceedCap);
}

#[test]
fn minter_can_update_minter_but_not_cap() {
    let app = App::default();

    let owner = "addr0001";
    let minter = "addr0002";
    let new_minter = "new_minter";
    let amount = Uint128::new(1234);
    let cap = Some(Uint128::from(3000000u128));

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
            mint: Some(MinterResponse {
                minter: minter.to_string(),
                cap,
            }),
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    // minter can mint coins to some winner
    contract
        .update_minter(Some(new_minter.to_string()))
        .call(minter)
        .unwrap();

    let resp = contract.minter().unwrap().unwrap();
    assert_eq!(
        resp,
        MinterResponse {
            minter: new_minter.to_string(),
            cap
        }
    );
}

#[test]
fn others_cannot_update_minter() {
    let app = App::default();

    let owner = "addr0001";
    let minter = "addr0002";
    let new_minter = "new_minter";
    let amount = Uint128::new(1234);

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
            mint: Some(MinterResponse {
                minter: minter.to_string(),
                cap: None,
            }),
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    let err = contract
        .update_minter(Some(new_minter.to_string()))
        .call(new_minter)
        .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized);
}

#[test]
fn unset_minter() {
    let app = App::default();

    let owner = "addr0001";
    let minter = "addr0002";
    let winner = "lucky";
    let amount = Uint128::new(1234);

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
            mint: Some(MinterResponse {
                minter: minter.to_string(),
                cap: None,
            }),
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    // Unset minter
    contract.update_minter(None).call(minter).unwrap();

    let resp = contract.minter().unwrap();
    assert_eq!(resp, None);

    // Old minter can no longer mint
    let err = contract
        .mint(winner.to_string(), amount)
        .call(minter)
        .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized);
}

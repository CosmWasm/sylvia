use cosmwasm_std::{Addr, StdError, Uint128};
use cw20_minting::responses::MinterResponse;
use cw_multi_test::App;

use crate::contract::InstantiateMsgData;
use crate::error::ContractError;
use crate::multitest::proxy::Cw20BaseCodeId;
use crate::responses::{Cw20Coin, TokenInfoResponse};

#[test]
fn mintable() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let minter = Addr::unchecked("addr0002");
    let amount = Uint128::new(11223344);
    let limit = Uint128::new(511223344);

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
                    address: owner.to_string(),
                    amount,
                }],
                mint: Some(MinterResponse {
                    minter: minter.to_string(),
                    cap: Some(limit),
                }),
                marketing: None,
            },
            "Cw20 contract",
        )
        .unwrap();

    // read token info
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

    // get owner balance
    let resp = contract.balance(&app, owner.to_string()).unwrap();

    assert_eq!(resp.balance, Uint128::new(11223344));

    // get minter balance
    let resp = contract.balance(&app, minter.to_string()).unwrap();

    assert_eq!(resp.balance, Uint128::new(0));

    // get minter
    let resp = contract.minter(&app).unwrap();

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
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let minter = Addr::unchecked("addr0002");
    let amount = Uint128::new(11223344);
    let limit = Uint128::new(11223300);

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let err = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
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
            },
            "Cw20 contract",
        )
        .unwrap_err();

    assert_eq!(
        err,
        StdError::generic_err("Initial supply greater than cap").into()
    );
}

#[test]
fn can_mint_by_minter() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let minter = Addr::unchecked("addr0002");
    let winner = Addr::unchecked("lucky");
    let prize = Uint128::new(222_222_222);
    let limit = Uint128::new(511223344);
    let amount = Uint128::new(11223344);

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
                    address: owner.to_string(),
                    amount,
                }],
                mint: Some(MinterResponse {
                    minter: minter.to_string(),
                    cap: Some(limit),
                }),
                marketing: None,
            },
            "Cw20 contract",
        )
        .unwrap();

    // minter can mint coins to some winner
    contract
        .mint(&mut app, &minter, winner.to_string(), prize)
        .unwrap();

    // but cannot mint nothing
    let err = contract
        .mint(&mut app, &minter, winner.to_string(), Uint128::zero())
        .unwrap_err();

    assert_eq!(err, ContractError::InvalidZeroAmount {});

    // but if it exceeds cap (even over multiple rounds), it fails
    // cap is enforced
    let err = contract
        .mint(
            &mut app,
            &minter,
            winner.to_string(),
            Uint128::new(333_222_222),
        )
        .unwrap_err();

    assert_eq!(err, ContractError::CannotExceedCap {});
}

#[test]
fn others_cannot_mint() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let minter = Addr::unchecked("addr0002");
    let winner = Addr::unchecked("lucky");
    let prize = Uint128::new(222_222_222);
    let limit = Uint128::new(511223344);
    let amount = Uint128::new(1234);

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
                    address: owner.to_string(),
                    amount,
                }],
                mint: Some(MinterResponse {
                    minter: minter.to_string(),
                    cap: Some(limit),
                }),
                marketing: None,
            },
            "Cw20 contract",
        )
        .unwrap();

    // minter can mint coins to some winner
    contract
        .mint(&mut app, &minter, winner.to_string(), prize)
        .unwrap();

    // but cannot mint nothing
    let err = contract
        .mint(&mut app, &minter, winner.to_string(), Uint128::zero())
        .unwrap_err();

    assert_eq!(err, ContractError::InvalidZeroAmount {});

    // but if it exceeds cap (even over multiple rounds), it fails
    // cap is enforced
    let err = contract
        .mint(
            &mut app,
            &minter,
            winner.to_string(),
            Uint128::new(333_222_222),
        )
        .unwrap_err();

    assert_eq!(err, ContractError::CannotExceedCap {});
}

#[test]
fn minter_can_update_minter_but_not_cap() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let minter = Addr::unchecked("addr0002");
    let new_minter = Addr::unchecked("new_minter");
    let amount = Uint128::new(1234);
    let cap = Some(Uint128::from(3000000u128));

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
                    address: owner.to_string(),
                    amount,
                }],
                mint: Some(MinterResponse {
                    minter: minter.to_string(),
                    cap,
                }),
                marketing: None,
            },
            "Cw20 contract",
        )
        .unwrap();

    // minter can mint coins to some winner
    contract
        .update_minter(&mut app, &minter, Some(new_minter.to_string()))
        .unwrap();

    let resp = contract.minter(&app).unwrap().unwrap();
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
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let minter = Addr::unchecked("addr0002");
    let new_minter = Addr::unchecked("new_minter");
    let amount = Uint128::new(1234);

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
                    address: owner.to_string(),
                    amount,
                }],
                mint: Some(MinterResponse {
                    minter: minter.to_string(),
                    cap: None,
                }),
                marketing: None,
            },
            "Cw20 contract",
        )
        .unwrap();

    let err = contract
        .update_minter(&mut app, &new_minter, Some(new_minter.to_string()))
        .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn unset_minter() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let minter = Addr::unchecked("addr0002");
    let winner = Addr::unchecked("lucky");
    let amount = Uint128::new(1234);

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
                    address: owner.to_string(),
                    amount,
                }],
                mint: Some(MinterResponse {
                    minter: minter.to_string(),
                    cap: None,
                }),
                marketing: None,
            },
            "Cw20 contract",
        )
        .unwrap();

    // Unset minter
    contract.update_minter(&mut app, &minter, None).unwrap();

    let resp = contract.minter(&app).unwrap();
    assert_eq!(resp, None);

    // Old minter can no longer mint
    let err = contract
        .mint(&mut app, &minter, winner.to_string(), amount)
        .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized {});
}

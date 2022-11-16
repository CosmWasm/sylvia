use cosmwasm_std::{Addr, Binary, StdError, Timestamp, Uint128};
use cw20_allowances::responses::{
    AllAllowancesResponse, AllSpenderAllowancesResponse, AllowanceInfo, AllowanceResponse,
    SpenderAllowanceInfo,
};
use cw_multi_test::{next_block, App};
use cw_utils::Expiration;

use crate::contract::InstantiateMsgData;
use crate::error::ContractError;
use crate::multitest::receiver_contract::ReceiverContractCodeId;
use crate::responses::Cw20Coin;

use super::proxy::Cw20BaseCodeId;

#[test]
fn increase_decrease_allowances() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let spender = Addr::unchecked("addr0002");

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Auto Gen".to_string(),
                symbol: "AUTO".to_string(),
                decimals: 3,
                initial_balances: vec![Cw20Coin {
                    address: owner.clone().into(),
                    amount: Uint128::new(12340000),
                }],
                mint: None,
                marketing: None,
            },
            "Cw20 contract",
        )
        .unwrap();

    // no allowance to start
    let allowances = contract
        .allowance(&app, owner.to_string(), spender.to_string())
        .unwrap();
    assert_eq!(allowances, AllowanceResponse::default());

    // set allowance with height expiration
    let allowance = Uint128::new(7777);
    let expires = Expiration::AtHeight(123_456);
    contract
        .increase_allowance(
            &mut app,
            &owner,
            spender.to_string(),
            allowance,
            Some(expires),
        )
        .unwrap();

    // ensure it looks good
    let allowance_resp = contract
        .allowance(&app, owner.to_string(), spender.to_string())
        .unwrap();

    assert_eq!(allowance_resp, AllowanceResponse { allowance, expires });

    // decrease it a bit with no expire set - stays the same
    let lower = Uint128::new(4444);
    let allowance = allowance.checked_sub(lower).unwrap();
    contract
        .decrease_allowance(&mut app, &owner, spender.to_string(), lower, None)
        .unwrap();

    let allowance_resp = contract
        .allowance(&app, owner.to_string(), spender.to_string())
        .unwrap();

    assert_eq!(allowance_resp, AllowanceResponse { allowance, expires });

    // increase it some more and override the expires
    let raise = Uint128::new(87654);
    let allowance = allowance + raise;
    let expires = Expiration::AtTime(Timestamp::from_seconds(8888888888));
    contract
        .increase_allowance(&mut app, &owner, spender.to_string(), raise, Some(expires))
        .unwrap();
    let allowance_resp = contract
        .allowance(&app, owner.to_string(), spender.to_string())
        .unwrap();
    assert_eq!(allowance_resp, AllowanceResponse { allowance, expires });

    // decrease it below 0
    contract
        .decrease_allowance(
            &mut app,
            &owner,
            spender.to_string(),
            Uint128::new(99988647623876347),
            None,
        )
        .unwrap();
    let allowance_resp = contract
        .allowance(&app, owner.to_string(), spender.to_string())
        .unwrap();
    assert_eq!(allowance_resp, AllowanceResponse::default());
}

#[test]
fn allowances_independent() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let spender = Addr::unchecked("addr0002");
    let spender2 = Addr::unchecked("addr0003");

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Auto Gen".to_string(),
                symbol: "AUTO".to_string(),
                decimals: 3,
                initial_balances: vec![Cw20Coin {
                    address: owner.clone().into(),
                    amount: Uint128::new(12340000),
                }],
                mint: None,
                marketing: None,
            },
            "Cw20 contract",
        )
        .unwrap();

    // no allowance to start
    let allowance_resp = contract
        .allowance(&app, owner.to_string(), spender.to_string())
        .unwrap();
    assert_eq!(allowance_resp, AllowanceResponse::default());

    let allowance_resp = contract
        .allowance(&app, owner.to_string(), spender2.to_string())
        .unwrap();
    assert_eq!(allowance_resp, AllowanceResponse::default());

    // set allowance with height expiration
    let allowance = Uint128::new(7777);
    let expires = Expiration::AtHeight(123_456);
    contract
        .increase_allowance(
            &mut app,
            &owner,
            spender.to_string(),
            allowance,
            Some(expires),
        )
        .unwrap();

    // set other allowance with no expiration
    let allowance2 = Uint128::new(87654);
    contract
        .increase_allowance(&mut app, &owner, spender2.to_string(), allowance2, None)
        .unwrap();

    // check they are proper
    let expect_one = AllowanceResponse { allowance, expires };
    let expect_two = AllowanceResponse {
        allowance: allowance2,
        expires: Expiration::Never {},
    };
    let allowance_resp = contract
        .allowance(&app, owner.to_string(), spender.to_string())
        .unwrap();
    assert_eq!(allowance_resp, expect_one);

    let allowance_resp = contract
        .allowance(&app, owner.to_string(), spender2.to_string())
        .unwrap();
    assert_eq!(allowance_resp, expect_two);

    // also allow spender -> spender2 with no interference
    let allowance3 = Uint128::new(1821);
    let expires3 = Expiration::AtTime(Timestamp::from_seconds(3767626296));
    contract
        .increase_allowance(
            &mut app,
            &Addr::unchecked(spender.to_string()),
            spender2.to_string(),
            allowance3,
            Some(expires3),
        )
        .unwrap();

    let expect_three = AllowanceResponse {
        allowance: allowance3,
        expires: expires3,
    };
    let allowance_resp = contract
        .allowance(&app, owner.to_string(), spender.to_string())
        .unwrap();
    assert_eq!(allowance_resp, expect_one);
    let allowance_resp = contract
        .allowance(&app, owner.to_string(), spender2.to_string())
        .unwrap();
    assert_eq!(allowance_resp, expect_two);
    let allowance_resp = contract
        .allowance(&app, spender.to_string(), spender2.to_string())
        .unwrap();
    assert_eq!(allowance_resp, expect_three);
}

#[test]
fn no_self_allowance() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Auto Gen".to_string(),
                symbol: "AUTO".to_string(),
                decimals: 3,
                initial_balances: vec![Cw20Coin {
                    address: owner.clone().into(),
                    amount: Uint128::new(12340000),
                }],
                mint: None,
                marketing: None,
            },
            "Cw20 contract",
        )
        .unwrap();

    // self-allowance
    let err = contract
        .increase_allowance(
            &mut app,
            &Addr::unchecked(owner.clone()),
            owner.to_string(),
            Uint128::new(7777),
            None,
        )
        .unwrap_err();

    assert_eq!(err, ContractError::CannotSetOwnAccount {});

    // decrease self-allowance
    let err = contract
        .decrease_allowance(
            &mut app,
            &Addr::unchecked(owner.clone()),
            owner.to_string(),
            Uint128::new(7777),
            None,
        )
        .unwrap_err();

    assert_eq!(err, ContractError::CannotSetOwnAccount {});
}

#[test]
fn transfer_from_self_to_self() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let amount = Uint128::new(999999);

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Auto Gen".to_string(),
                symbol: "AUTO".to_string(),
                decimals: 3,
                initial_balances: vec![Cw20Coin {
                    address: owner.clone().into(),
                    amount,
                }],
                mint: None,
                marketing: None,
            },
            "Cw20 contract",
        )
        .unwrap();

    // valid transfer of part of the allowance
    let transfer = Uint128::new(44444);
    contract
        .transfer_from(
            &mut app,
            &owner,
            owner.to_string(),
            owner.to_string(),
            transfer,
        )
        .unwrap();

    // make sure amount of money is the same
    let balance_resp = contract.balance(&app, owner.to_string()).unwrap();
    assert_eq!(balance_resp.balance, amount);
}

#[test]
fn transfer_from_owner_requires_no_allowance() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let rcpt = Addr::unchecked("addr0003");
    let start_amount = Uint128::new(999999);

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Auto Gen".to_string(),
                symbol: "AUTO".to_string(),
                decimals: 3,
                initial_balances: vec![Cw20Coin {
                    address: owner.clone().into(),
                    amount: start_amount,
                }],
                mint: None,
                marketing: None,
            },
            "Cw20 contract",
        )
        .unwrap();

    // valid transfer of part of the allowance
    let transfer = Uint128::new(44444);
    contract
        .transfer_from(
            &mut app,
            &owner,
            owner.to_string(),
            rcpt.to_string(),
            transfer,
        )
        .unwrap();

    // make sure money arrived
    let balance_resp = contract.balance(&app, owner.to_string()).unwrap();
    assert_eq!(
        balance_resp.balance,
        start_amount.checked_sub(transfer).unwrap()
    );

    let balance_resp = contract.balance(&app, rcpt.to_string()).unwrap();
    assert_eq!(balance_resp.balance, transfer);
}

#[test]
fn transfer_from_respects_limits() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let spender = Addr::unchecked("addr0002");
    let rcpt = Addr::unchecked("addr0003");
    let start_amount = Uint128::new(999999);

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Auto Gen".to_string(),
                symbol: "AUTO".to_string(),
                decimals: 3,
                initial_balances: vec![Cw20Coin {
                    address: owner.clone().into(),
                    amount: start_amount,
                }],
                mint: None,
                marketing: None,
            },
            "Cw20 contract",
        )
        .unwrap();

    // provide an allowance
    let allowance = Uint128::new(77777);
    contract
        .increase_allowance(&mut app, &owner, spender.to_string(), allowance, None)
        .unwrap();

    // valid transfer of part of the allowance
    let transfer = Uint128::new(44444);
    contract
        .transfer_from(
            &mut app,
            &Addr::unchecked(spender.to_string()),
            owner.to_string(),
            rcpt.to_string(),
            transfer,
        )
        .unwrap();

    // make sure money arrived
    let balance_resp = contract.balance(&app, owner.to_string()).unwrap();
    assert_eq!(
        balance_resp.balance,
        start_amount.checked_sub(transfer).unwrap()
    );

    let balance_resp = contract.balance(&app, rcpt.to_string()).unwrap();
    assert_eq!(balance_resp.balance, transfer);

    // ensure it looks good
    let allowance_resp = contract
        .allowance(&app, owner.to_string(), spender.to_string())
        .unwrap();
    assert_eq!(
        allowance_resp,
        AllowanceResponse {
            allowance: allowance.checked_sub(transfer).unwrap(),
            expires: Expiration::Never {}
        }
    );

    // cannot send more than the allowance
    let err = contract
        .transfer_from(
            &mut app,
            &Addr::unchecked(spender.to_string()),
            owner.to_string(),
            rcpt.to_string(),
            Uint128::new(33443),
        )
        .unwrap_err();
    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // let us increase limit, but set the expiration to expire in the next block
    let next_block_height = app.block_info().height + 1;
    contract
        .increase_allowance(
            &mut app,
            &owner,
            spender.to_string(),
            Uint128::new(1000),
            Some(Expiration::AtHeight(next_block_height)),
        )
        .unwrap();

    // move to next block
    app.update_block(next_block);

    // we should now get the expiration error
    let err = contract
        .transfer_from(
            &mut app,
            &spender,
            owner.to_string(),
            rcpt.to_string(),
            Uint128::new(33443),
        )
        .unwrap_err();
    assert!(matches!(err, ContractError::Expired {}));
}

#[test]
fn burn_from_respects_limits() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let spender = Addr::unchecked("addr0002");
    let start_amount = Uint128::new(999999);

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Auto Gen".to_string(),
                symbol: "AUTO".to_string(),
                decimals: 3,
                initial_balances: vec![Cw20Coin {
                    address: owner.clone().into(),
                    amount: start_amount,
                }],
                mint: None,
                marketing: None,
            },
            "Cw20 contract",
        )
        .unwrap();

    // provide an allowance
    let allowance = Uint128::new(77777);
    contract
        .increase_allowance(&mut app, &owner, spender.to_string(), allowance, None)
        .unwrap();

    // valid burn of part of the allowance
    let transfer = Uint128::new(44444);
    contract
        .burn_from(&mut app, &spender, owner.to_string(), transfer)
        .unwrap();

    // make sure money burnt
    let balance_resp = contract.balance(&app, owner.to_string()).unwrap();
    assert_eq!(
        balance_resp.balance,
        start_amount.checked_sub(transfer).unwrap()
    );

    // ensure it looks good
    let allowance_resp = contract
        .allowance(&app, owner.to_string(), spender.to_string())
        .unwrap();

    assert_eq!(
        allowance_resp,
        AllowanceResponse {
            allowance: allowance.checked_sub(transfer).unwrap(),
            expires: Expiration::Never {}
        }
    );

    // cannot burn more than the allowance
    let err = contract
        .burn_from(&mut app, &spender, owner.to_string(), Uint128::new(33443))
        .unwrap_err();

    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // let us increase limit, but set the expiration to expire in the next block
    let next_block_height = app.block_info().height + 1;
    contract
        .increase_allowance(
            &mut app,
            &owner,
            spender.to_string(),
            Uint128::new(1000),
            Some(Expiration::AtHeight(next_block_height)),
        )
        .unwrap();

    // move to next block
    app.update_block(next_block);

    // we should now get the expiration error
    let err = contract
        .burn_from(&mut app, &spender, owner.to_string(), Uint128::new(33443))
        .unwrap_err();
    assert!(matches!(err, ContractError::Expired {}));
}

// Ignoring currently due to some issue with unsupported msg being sent in send_from
#[test]
fn send_from_respects_limits() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let owner2 = Addr::unchecked("addr0003");
    let spender = Addr::unchecked("addr0002");
    let send_msg = Binary::from(r#"{"some":123}"#.as_bytes());
    let start_amount = Uint128::new(999999);

    let code_id = Cw20BaseCodeId::store_code(&mut app);
    let receiver_code_id = ReceiverContractCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Auto Gen".to_string(),
                symbol: "AUTO".to_string(),
                decimals: 3,
                initial_balances: vec![Cw20Coin {
                    address: owner.clone().into(),
                    amount: start_amount,
                }],
                mint: None,
                marketing: None,
            },
            "Cw20 contract",
        )
        .unwrap();

    let receiver_contract = receiver_code_id
        .instantiate(&mut app, &owner2, "cool-dex")
        .unwrap();

    // provide an allowance
    let allowance = Uint128::new(77777);
    contract
        .increase_allowance(&mut app, &owner, spender.to_string(), allowance, None)
        .unwrap();

    // valid send of part of the allowance
    let transfer = Uint128::new(44444);
    contract
        .send_from(
            &mut app,
            &spender,
            owner.to_string(),
            receiver_contract.addr().to_string(),
            transfer,
            send_msg.clone(),
        )
        .unwrap();

    // make sure money burnt
    let balance_resp = contract.balance(&app, owner.to_string()).unwrap();
    assert_eq!(
        balance_resp.balance,
        start_amount.checked_sub(transfer).unwrap()
    );

    // ensure it looks good
    let allowance_resp = contract
        .allowance(&app, owner.to_string(), spender.to_string())
        .unwrap();

    assert_eq!(
        allowance_resp,
        AllowanceResponse {
            allowance: allowance.checked_sub(transfer).unwrap(),
            expires: Expiration::Never {}
        }
    );

    // cannot send more than the allowance
    let err = contract
        .send_from(
            &mut app,
            &spender,
            owner.to_string(),
            receiver_contract.addr().to_string(),
            Uint128::new(33443),
            send_msg.clone(),
        )
        .unwrap_err();
    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // let us increase limit, but set the expiration to expire in the next block
    let next_block_height = app.block_info().height + 1;
    contract
        .increase_allowance(
            &mut app,
            &owner,
            spender.to_string(),
            Uint128::new(1000),
            Some(Expiration::AtHeight(next_block_height)),
        )
        .unwrap();

    // move to next block
    app.update_block(next_block);

    // we should now get the expiration error
    let err = contract
        .send_from(
            &mut app,
            &spender,
            owner.to_string(),
            receiver_contract.addr().to_string(),
            Uint128::new(33443),
            send_msg,
        )
        .unwrap_err();

    assert!(matches!(err, ContractError::Expired {}));
}

#[test]
fn no_past_expiration() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let spender = Addr::unchecked("addr0002");
    let start_amount = Uint128::new(999999);
    let allowance = Uint128::new(7777);

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Auto Gen".to_string(),
                symbol: "AUTO".to_string(),
                decimals: 3,
                initial_balances: vec![Cw20Coin {
                    address: owner.clone().into(),
                    amount: start_amount,
                }],
                mint: None,
                marketing: None,
            },
            "Cw20 contract",
        )
        .unwrap();

    // set allowance with height expiration at current block height
    let block_height = app.block_info().height;
    let expires = Expiration::AtHeight(block_height);

    let err = contract
        .increase_allowance(
            &mut app,
            &owner,
            spender.to_string(),
            allowance,
            Some(expires),
        )
        .unwrap_err();

    // ensure it is rejected
    assert_eq!(ContractError::InvalidExpiration {}, err);

    // set allowance with time expiration in the past
    let block_time = app.block_info().time;
    let expires = Expiration::AtTime(block_time.minus_seconds(1));

    let err = contract
        .increase_allowance(
            &mut app,
            &owner,
            spender.to_string(),
            allowance,
            Some(expires),
        )
        .unwrap_err();

    // ensure it is rejected
    assert_eq!(ContractError::InvalidExpiration {}, err);

    // set allowance with height expiration at next block height
    let block_height = app.block_info().height + 1;
    let expires = Expiration::AtHeight(block_height);

    contract
        .increase_allowance(
            &mut app,
            &owner,
            spender.to_string(),
            allowance,
            Some(expires),
        )
        .unwrap();

    // ensure it looks good
    let allowance_resp = contract
        .allowance(&app, owner.to_string(), spender.to_string())
        .unwrap();

    assert_eq!(allowance_resp, AllowanceResponse { allowance, expires });

    // set allowance with time expiration in the future
    let block_time = app.block_info().time;
    let expires = Expiration::AtTime(block_time.plus_seconds(10));

    contract
        .increase_allowance(
            &mut app,
            &owner,
            spender.to_string(),
            allowance,
            Some(expires),
        )
        .unwrap();

    // ensure it looks good
    let allowance_resp = contract
        .allowance(&app, owner.to_string(), spender.to_string())
        .unwrap();

    assert_eq!(
        allowance_resp,
        AllowanceResponse {
            allowance: allowance + allowance, // we increased twice
            expires
        }
    );

    // decrease with height expiration at current block height
    let block_height = app.block_info().height;
    let expires = Expiration::AtHeight(block_height);

    let err = contract
        .increase_allowance(
            &mut app,
            &owner,
            spender.to_string(),
            allowance,
            Some(expires),
        )
        .unwrap_err();

    // ensure it is rejected
    assert_eq!(ContractError::InvalidExpiration {}, err);

    // decrease with height expiration at next block height
    let block_height = app.block_info().height + 1;
    let expires = Expiration::AtHeight(block_height);

    contract
        .decrease_allowance(
            &mut app,
            &owner,
            spender.to_string(),
            allowance,
            Some(expires),
        )
        .unwrap();

    // ensure it looks good
    let allowance_resp = contract
        .allowance(&app, owner.to_string(), spender.to_string())
        .unwrap();

    assert_eq!(allowance_resp, AllowanceResponse { allowance, expires });
}

#[test]
fn query_allowances() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let spender = Addr::unchecked("addr0002");
    let spender2 = Addr::unchecked("addr0003");
    let start_amount = Uint128::new(999999);
    let allowance = Uint128::new(7777);

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Auto Gen".to_string(),
                symbol: "AUTO".to_string(),
                decimals: 3,
                initial_balances: vec![Cw20Coin {
                    address: owner.clone().into(),
                    amount: start_amount,
                }],
                mint: None,
                marketing: None,
            },
            "Cw20 contract",
        )
        .unwrap();

    // check all allowances
    let all_allowances_resp = contract
        .all_allowances(&app, owner.to_string(), None, None)
        .unwrap();

    assert_eq!(
        all_allowances_resp,
        AllAllowancesResponse { allowances: vec![] }
    );

    // increase spender allowances
    contract
        .increase_allowance(&mut app, &owner, spender.to_string(), allowance, None)
        .unwrap();

    // check all allowances
    let all_allowances_resp = contract
        .all_allowances(&app, owner.to_string(), None, None)
        .unwrap();

    assert_eq!(
        all_allowances_resp,
        AllAllowancesResponse {
            allowances: vec![AllowanceInfo {
                spender: spender.to_string(),
                allowance,
                expires: Expiration::Never {}
            }]
        }
    );

    // check spender allowances
    let all_spender_allowances_resp = contract
        .all_spender_allowances(&app, spender.to_string(), None, None)
        .unwrap();

    assert_eq!(
        all_spender_allowances_resp,
        AllSpenderAllowancesResponse {
            allowances: vec![SpenderAllowanceInfo {
                owner: owner.to_string(),
                allowance,
                expires: Expiration::Never {}
            }]
        }
    );

    // increase spender2 allowances
    let increased_allowances = allowance + allowance;
    contract
        .increase_allowance(
            &mut app,
            &owner,
            spender2.to_string(),
            increased_allowances,
            None,
        )
        .unwrap();

    // check all allowances
    let all_allowances_resp = contract
        .all_allowances(&app, owner.to_string(), None, None)
        .unwrap();

    assert_eq!(
        all_allowances_resp,
        AllAllowancesResponse {
            allowances: vec![
                AllowanceInfo {
                    spender: spender.to_string(),
                    allowance,
                    expires: Expiration::Never {}
                },
                AllowanceInfo {
                    spender: spender2.to_string(),
                    allowance: increased_allowances,
                    expires: Expiration::Never {}
                }
            ]
        }
    );

    // check all allowances with limit
    let all_allowances_resp = contract
        .all_allowances(&app, owner.to_string(), None, Some(1))
        .unwrap();

    assert_eq!(
        all_allowances_resp,
        AllAllowancesResponse {
            allowances: vec![AllowanceInfo {
                spender: spender.to_string(),
                allowance,
                expires: Expiration::Never {}
            },]
        }
    );
}

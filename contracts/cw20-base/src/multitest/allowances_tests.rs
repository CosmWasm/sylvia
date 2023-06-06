use cosmwasm_std::{Binary, StdError, Timestamp, Uint128};
use cw20_allowances::responses::{
    AllAllowancesResponse, AllSpenderAllowancesResponse, AllowanceInfo, AllowanceResponse,
    SpenderAllowanceInfo,
};
use cw_multi_test::next_block;
use cw_utils::Expiration;
use sylvia::multitest::App;

use crate::allowances::test_utils::Cw20Allowances;
use crate::contract::multitest_utils::CodeId;
use crate::contract::InstantiateMsgData;
use crate::error::ContractError;
use crate::multitest::receiver_contract::multitest_utils::CodeId as ReceiverCodeId;
use crate::responses::Cw20Coin;

#[test]
fn increase_decrease_allowances() {
    let app = App::default();

    let owner = "addr0001";
    let spender = "addr0002";

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Auto Gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: owner.into(),
                amount: Uint128::new(12340000),
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    // no allowance to start
    let allowances = contract
        .cw20_allowances_proxy()
        .allowance(owner.to_string(), spender.to_string())
        .unwrap();
    assert_eq!(allowances, AllowanceResponse::default());

    // set allowance with height expiration
    let allowance = Uint128::new(7777);
    let expires = Expiration::AtHeight(123_456);
    contract
        .cw20_allowances_proxy()
        .increase_allowance(spender.to_string(), allowance, Some(expires))
        .call(owner)
        .unwrap();

    // ensure it looks good
    let allowance_resp = contract
        .cw20_allowances_proxy()
        .allowance(owner.to_string(), spender.to_string())
        .unwrap();

    assert_eq!(allowance_resp, AllowanceResponse { allowance, expires });

    // decrease it a bit with no expire set - stays the same
    let lower = Uint128::new(4444);
    let allowance = allowance.checked_sub(lower).unwrap();
    contract
        .cw20_allowances_proxy()
        .decrease_allowance(spender.to_string(), lower, None)
        .call(owner)
        .unwrap();

    let allowance_resp = contract
        .cw20_allowances_proxy()
        .allowance(owner.to_string(), spender.to_string())
        .unwrap();

    assert_eq!(allowance_resp, AllowanceResponse { allowance, expires });

    // increase it some more and override the expires
    let raise = Uint128::new(87654);
    let allowance = allowance + raise;
    let expires = Expiration::AtTime(Timestamp::from_seconds(8888888888));
    contract
        .cw20_allowances_proxy()
        .increase_allowance(spender.to_string(), raise, Some(expires))
        .call(owner)
        .unwrap();

    let allowance_resp = contract
        .cw20_allowances_proxy()
        .allowance(owner.to_string(), spender.to_string())
        .unwrap();
    assert_eq!(allowance_resp, AllowanceResponse { allowance, expires });

    // decrease it below 0
    contract
        .cw20_allowances_proxy()
        .decrease_allowance(spender.to_string(), Uint128::new(99988647623876347), None)
        .call(owner)
        .unwrap();

    let allowance_resp = contract
        .cw20_allowances_proxy()
        .allowance(owner.to_string(), spender.to_string())
        .unwrap();
    assert_eq!(allowance_resp, AllowanceResponse::default());
}

#[test]
fn allowances_independent() {
    let app = App::default();

    let owner = "addr0001";
    let spender = "addr0002";
    let spender2 = "addr0003";

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Auto Gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: owner.into(),
                amount: Uint128::new(12340000),
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    // no allowance to start
    let allowance_resp = contract
        .cw20_allowances_proxy()
        .allowance(owner.to_string(), spender.to_string())
        .unwrap();
    assert_eq!(allowance_resp, AllowanceResponse::default());

    let allowance_resp = contract
        .cw20_allowances_proxy()
        .allowance(owner.to_string(), spender2.to_string())
        .unwrap();
    assert_eq!(allowance_resp, AllowanceResponse::default());

    // set allowance with height expiration
    let allowance = Uint128::new(7777);
    let expires = Expiration::AtHeight(123_456);
    contract
        .cw20_allowances_proxy()
        .increase_allowance(spender.to_string(), allowance, Some(expires))
        .call(owner)
        .unwrap();

    // set other allowance with no expiration
    let allowance2 = Uint128::new(87654);
    contract
        .cw20_allowances_proxy()
        .increase_allowance(spender2.to_string(), allowance2, None)
        .call(owner)
        .unwrap();

    // check they are proper
    let expect_one = AllowanceResponse { allowance, expires };
    let expect_two = AllowanceResponse {
        allowance: allowance2,
        expires: Expiration::Never {},
    };
    let allowance_resp = contract
        .cw20_allowances_proxy()
        .allowance(owner.to_string(), spender.to_string())
        .unwrap();
    assert_eq!(allowance_resp, expect_one);

    let allowance_resp = contract
        .cw20_allowances_proxy()
        .allowance(owner.to_string(), spender2.to_string())
        .unwrap();
    assert_eq!(allowance_resp, expect_two);

    // also allow spender -> spender2 with no interference
    let allowance3 = Uint128::new(1821);
    let expires3 = Expiration::AtTime(Timestamp::from_seconds(3767626296));
    contract
        .cw20_allowances_proxy()
        .increase_allowance(spender2.to_string(), allowance3, Some(expires3))
        .call(spender)
        .unwrap();

    let expect_three = AllowanceResponse {
        allowance: allowance3,
        expires: expires3,
    };
    let allowance_resp = contract
        .cw20_allowances_proxy()
        .allowance(owner.to_string(), spender.to_string())
        .unwrap();
    assert_eq!(allowance_resp, expect_one);
    let allowance_resp = contract
        .cw20_allowances_proxy()
        .allowance(owner.to_string(), spender2.to_string())
        .unwrap();
    assert_eq!(allowance_resp, expect_two);
    let allowance_resp = contract
        .cw20_allowances_proxy()
        .allowance(spender.to_string(), spender2.to_string())
        .unwrap();
    assert_eq!(allowance_resp, expect_three);
}

#[test]
fn no_self_allowance() {
    let app = App::default();

    let owner = "addr0001";

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Auto Gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: owner.into(),
                amount: Uint128::new(12340000),
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    // self-allowance
    let err = contract
        .cw20_allowances_proxy()
        .increase_allowance(owner.to_string(), Uint128::new(7777), None)
        .call(owner)
        .unwrap_err();

    assert_eq!(err, ContractError::CannotSetOwnAccount);

    // decrease self-allowance
    let err = contract
        .cw20_allowances_proxy()
        .decrease_allowance(owner.to_string(), Uint128::new(7777), None)
        .call(owner)
        .unwrap_err();

    assert_eq!(err, ContractError::CannotSetOwnAccount);
}

#[test]
fn transfer_from_self_to_self() {
    let app = App::default();

    let owner = "addr0001";
    let amount = Uint128::new(999999);

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Auto Gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: owner.into(),
                amount,
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    // valid transfer of part of the allowance
    let transfer = Uint128::new(44444);
    contract
        .cw20_allowances_proxy()
        .transfer_from(owner.to_string(), owner.to_string(), transfer)
        .call(owner)
        .unwrap();

    // make sure amount of money is the same
    let balance_resp = contract.balance(owner.to_string()).unwrap();
    assert_eq!(balance_resp.balance, amount);
}

#[test]
fn transfer_from_owner_requires_no_allowance() {
    let app = App::default();

    let owner = "addr0001";
    let rcpt = "addr0003";
    let start_amount = Uint128::new(999999);

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Auto Gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: owner.into(),
                amount: start_amount,
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    // valid transfer of part of the allowance
    let transfer = Uint128::new(44444);
    contract
        .cw20_allowances_proxy()
        .transfer_from(owner.to_string(), rcpt.to_string(), transfer)
        .call(owner)
        .unwrap();

    // make sure money arrived
    let balance_resp = contract.balance(owner.to_string()).unwrap();
    assert_eq!(
        balance_resp.balance,
        start_amount.checked_sub(transfer).unwrap()
    );

    let balance_resp = contract.balance(rcpt.to_string()).unwrap();
    assert_eq!(balance_resp.balance, transfer);
}

#[test]
fn transfer_from_respects_limits() {
    let app = App::default();

    let owner = "addr0001";
    let spender = "addr0002";
    let rcpt = "addr0003";
    let start_amount = Uint128::new(999999);

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Auto Gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: owner.into(),
                amount: start_amount,
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    // provide an allowance
    let allowance = Uint128::new(77777);
    contract
        .cw20_allowances_proxy()
        .increase_allowance(spender.to_string(), allowance, None)
        .call(owner)
        .unwrap();

    // valid transfer of part of the allowance
    let transfer = Uint128::new(44444);
    contract
        .cw20_allowances_proxy()
        .transfer_from(owner.to_string(), rcpt.to_string(), transfer)
        .call(spender)
        .unwrap();

    // make sure money arrived
    let balance_resp = contract.balance(owner.to_string()).unwrap();
    assert_eq!(
        balance_resp.balance,
        start_amount.checked_sub(transfer).unwrap()
    );

    let balance_resp = contract.balance(rcpt.to_string()).unwrap();
    assert_eq!(balance_resp.balance, transfer);

    // ensure it looks good
    let allowance_resp = contract
        .cw20_allowances_proxy()
        .allowance(owner.to_string(), spender.to_string())
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
        .cw20_allowances_proxy()
        .transfer_from(owner.to_string(), rcpt.to_string(), Uint128::new(33443))
        .call(spender)
        .unwrap_err();
    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // let us increase limit, but set the expiration to expire in the next block
    let next_block_height = (*app).borrow().block_info().height + 1;
    contract
        .cw20_allowances_proxy()
        .increase_allowance(
            spender.to_string(),
            Uint128::new(1000),
            Some(Expiration::AtHeight(next_block_height)),
        )
        .call(owner)
        .unwrap();

    // move to next block
    (*app).borrow_mut().update_block(next_block);

    // we should now get the expiration error
    let err = contract
        .cw20_allowances_proxy()
        .transfer_from(owner.to_string(), rcpt.to_string(), Uint128::new(33443))
        .call(spender)
        .unwrap_err();
    assert!(matches!(err, ContractError::Expired));
}

#[test]
fn burn_from_respects_limits() {
    let app = App::default();

    let owner = "addr0001";
    let spender = "addr0002";
    let start_amount = Uint128::new(999999);

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Auto Gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: owner.into(),
                amount: start_amount,
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    // provide an allowance
    let allowance = Uint128::new(77777);
    contract
        .cw20_allowances_proxy()
        .increase_allowance(spender.to_string(), allowance, None)
        .call(owner)
        .unwrap();

    // valid burn of part of the allowance
    let transfer = Uint128::new(44444);
    contract
        .cw20_allowances_proxy()
        .burn_from(owner.to_string(), transfer)
        .call(spender)
        .unwrap();

    // make sure money burnt
    let balance_resp = contract.balance(owner.to_string()).unwrap();
    assert_eq!(
        balance_resp.balance,
        start_amount.checked_sub(transfer).unwrap()
    );

    // ensure it looks good
    let allowance_resp = contract
        .cw20_allowances_proxy()
        .allowance(owner.to_string(), spender.to_string())
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
        .cw20_allowances_proxy()
        .burn_from(owner.to_string(), Uint128::new(33443))
        .call(spender)
        .unwrap_err();

    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // let us increase limit, but set the expiration to expire in the next block
    let next_block_height = (*app).borrow().block_info().height + 1;
    contract
        .cw20_allowances_proxy()
        .increase_allowance(
            spender.to_string(),
            Uint128::new(1000),
            Some(Expiration::AtHeight(next_block_height)),
        )
        .call(owner)
        .unwrap();

    // move to next block
    (*app).borrow_mut().update_block(next_block);

    // we should now get the expiration error
    let err = contract
        .cw20_allowances_proxy()
        .burn_from(owner.to_string(), Uint128::new(33443))
        .call(spender)
        .unwrap_err();
    assert!(matches!(err, ContractError::Expired));
}

// Ignoring currently due to some issue with unsupported msg being sent in send_from
#[test]
fn send_from_respects_limits() {
    let app = App::default();

    let owner = "addr0001";
    let owner2 = "addr0003";
    let spender = "addr0002";
    let send_msg = Binary::from(r#"{"some":123}"#.as_bytes());
    let start_amount = Uint128::new(999999);

    let code_id = CodeId::store_code(&app);
    let receiver_code_id = ReceiverCodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Auto Gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: owner.into(),
                amount: start_amount,
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    let receiver_contract = receiver_code_id
        .instantiate()
        .with_label("cool-dex")
        .call(owner2)
        .unwrap();

    // provide an allowance
    let allowance = Uint128::new(77777);
    contract
        .cw20_allowances_proxy()
        .increase_allowance(spender.to_string(), allowance, None)
        .call(owner)
        .unwrap();

    // valid send of part of the allowance
    let transfer = Uint128::new(44444);
    contract
        .cw20_allowances_proxy()
        .send_from(
            owner.to_string(),
            receiver_contract.contract_addr.to_string(),
            transfer,
            send_msg.clone(),
        )
        .call(spender)
        .unwrap();

    // make sure money burnt
    let balance_resp = contract.balance(owner.to_string()).unwrap();
    assert_eq!(
        balance_resp.balance,
        start_amount.checked_sub(transfer).unwrap()
    );

    // ensure it looks good
    let allowance_resp = contract
        .cw20_allowances_proxy()
        .allowance(owner.to_string(), spender.to_string())
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
        .cw20_allowances_proxy()
        .send_from(
            owner.to_string(),
            receiver_contract.contract_addr.to_string(),
            Uint128::new(33443),
            send_msg.clone(),
        )
        .call(spender)
        .unwrap_err();
    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // let us increase limit, but set the expiration to expire in the next block
    let next_block_height = (*app).borrow().block_info().height + 1;
    contract
        .cw20_allowances_proxy()
        .increase_allowance(
            spender.to_string(),
            Uint128::new(1000),
            Some(Expiration::AtHeight(next_block_height)),
        )
        .call(owner)
        .unwrap();

    // move to next block
    (*app).borrow_mut().update_block(next_block);

    // we should now get the expiration error
    let err = contract
        .cw20_allowances_proxy()
        .send_from(
            owner.to_string(),
            receiver_contract.contract_addr.to_string(),
            Uint128::new(33443),
            send_msg,
        )
        .call(spender)
        .unwrap_err();

    assert!(matches!(err, ContractError::Expired));
}

#[test]
fn no_past_expiration() {
    let app = App::default();

    let owner = "addr0001";
    let spender = "addr0002";
    let start_amount = Uint128::new(999999);
    let allowance = Uint128::new(7777);

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Auto Gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: owner.into(),
                amount: start_amount,
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    // set allowance with height expiration at current block height
    let block_height = (*app).borrow().block_info().height;
    let expires = Expiration::AtHeight(block_height);

    let err = contract
        .cw20_allowances_proxy()
        .increase_allowance(spender.to_string(), allowance, Some(expires))
        .call(owner)
        .unwrap_err();

    // ensure it is rejected
    assert_eq!(ContractError::InvalidExpiration, err);

    // set allowance with time expiration in the past
    let block_time = (*app).borrow().block_info().time;
    let expires = Expiration::AtTime(block_time.minus_seconds(1));

    let err = contract
        .cw20_allowances_proxy()
        .increase_allowance(spender.to_string(), allowance, Some(expires))
        .call(owner)
        .unwrap_err();

    // ensure it is rejected
    assert_eq!(ContractError::InvalidExpiration, err);

    // set allowance with height expiration at next block height
    let block_height = (*app).borrow().block_info().height + 1;
    let expires = Expiration::AtHeight(block_height);

    contract
        .cw20_allowances_proxy()
        .increase_allowance(spender.to_string(), allowance, Some(expires))
        .call(owner)
        .unwrap();

    // ensure it looks good
    let allowance_resp = contract
        .cw20_allowances_proxy()
        .allowance(owner.to_string(), spender.to_string())
        .unwrap();

    assert_eq!(allowance_resp, AllowanceResponse { allowance, expires });

    // set allowance with time expiration in the future
    let block_time = (*app).borrow().block_info().time;
    let expires = Expiration::AtTime(block_time.plus_seconds(10));

    contract
        .cw20_allowances_proxy()
        .increase_allowance(spender.to_string(), allowance, Some(expires))
        .call(owner)
        .unwrap();

    // ensure it looks good
    let allowance_resp = contract
        .cw20_allowances_proxy()
        .allowance(owner.to_string(), spender.to_string())
        .unwrap();

    assert_eq!(
        allowance_resp,
        AllowanceResponse {
            allowance: allowance + allowance, // we increased twice
            expires
        }
    );

    // decrease with height expiration at current block height
    let block_height = (*app).borrow().block_info().height;
    let expires = Expiration::AtHeight(block_height);

    let err = contract
        .cw20_allowances_proxy()
        .increase_allowance(spender.to_string(), allowance, Some(expires))
        .call(owner)
        .unwrap_err();

    // ensure it is rejected
    assert_eq!(ContractError::InvalidExpiration, err);

    // decrease with height expiration at next block height
    let block_height = (*app).borrow().block_info().height + 1;
    let expires = Expiration::AtHeight(block_height);

    contract
        .cw20_allowances_proxy()
        .decrease_allowance(spender.to_string(), allowance, Some(expires))
        .call(owner)
        .unwrap();

    // ensure it looks good
    let allowance_resp = contract
        .cw20_allowances_proxy()
        .allowance(owner.to_string(), spender.to_string())
        .unwrap();

    assert_eq!(allowance_resp, AllowanceResponse { allowance, expires });
}

#[test]
fn query_allowances() {
    let app = App::default();

    let owner = "addr0001";
    let spender = "addr0002";
    let spender2 = "addr0003";
    let start_amount = Uint128::new(999999);
    let allowance = Uint128::new(7777);

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Auto Gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: owner.into(),
                amount: start_amount,
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    // check all allowances
    let all_allowances_resp = contract
        .cw20_allowances_proxy()
        .all_allowances(owner.to_string(), None, None)
        .unwrap();

    assert_eq!(
        all_allowances_resp,
        AllAllowancesResponse { allowances: vec![] }
    );

    // increase spender allowances
    contract
        .cw20_allowances_proxy()
        .increase_allowance(spender.to_string(), allowance, None)
        .call(owner)
        .unwrap();

    // check all allowances
    let all_allowances_resp = contract
        .cw20_allowances_proxy()
        .all_allowances(owner.to_string(), None, None)
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
        .cw20_allowances_proxy()
        .all_spender_allowances(spender.to_string(), None, None)
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
        .cw20_allowances_proxy()
        .increase_allowance(spender2.to_string(), increased_allowances, None)
        .call(owner)
        .unwrap();

    // check all allowances
    let all_allowances_resp = contract
        .cw20_allowances_proxy()
        .all_allowances(owner.to_string(), None, None)
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
        .cw20_allowances_proxy()
        .all_allowances(owner.to_string(), None, Some(1))
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

#[test]
fn query_all_allowances_works() {
    let app = App::default();

    let owner = "addr0001";
    let spender = "addr0002";
    let spender2 = "addr0003";
    let start_amount = Uint128::new(12340000);

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Auto Gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: owner.into(),
                amount: start_amount,
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    // no allowance to start
    let resp = contract
        .cw20_allowances_proxy()
        .all_allowances(owner.to_string(), None, None)
        .unwrap();
    assert_eq!(resp.allowances, vec![]);

    // set allowance with height expiration
    let allow1 = Uint128::new(7777);
    let expires = Expiration::AtHeight(123_456);
    contract
        .cw20_allowances_proxy()
        .increase_allowance(spender.to_string(), allow1, Some(expires))
        .call(owner)
        .unwrap();

    // set allowance with no expiration
    let allow2 = Uint128::new(54321);
    contract
        .cw20_allowances_proxy()
        .increase_allowance(spender2.to_string(), allow2, None)
        .call(owner)
        .unwrap();

    // query list gets 2
    let resp = contract
        .cw20_allowances_proxy()
        .all_allowances(owner.to_string(), None, None)
        .unwrap();
    assert_eq!(resp.allowances.len(), 2);

    // first one is spender1 (order of CanonicalAddr uncorrelated with String)
    let resp = contract
        .cw20_allowances_proxy()
        .all_allowances(owner.to_string(), None, Some(1))
        .unwrap();
    assert_eq!(
        resp,
        AllAllowancesResponse {
            allowances: vec![AllowanceInfo {
                spender: spender.to_string(),
                allowance: allow1,
                expires
            }]
        }
    );

    // next one is spender2
    let resp = contract
        .cw20_allowances_proxy()
        .all_allowances(owner.to_string(), Some(spender.to_string()), Some(10000))
        .unwrap();
    assert_eq!(
        resp,
        AllAllowancesResponse {
            allowances: vec![AllowanceInfo {
                spender: spender2.to_string(),
                allowance: allow2,
                expires: Expiration::Never {}
            }]
        }
    );
}

#[test]
fn all_spender_allowances_on_two_contracts() {
    let app = App::default();

    let owner = "addr0001";
    let owner2 = "addr0003";
    let spender = "addr0002";
    let start_amount = Uint128::new(12340000);

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Auto Gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: owner.into(),
                amount: start_amount,
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    // no allowance to start
    let resp = contract
        .cw20_allowances_proxy()
        .all_spender_allowances(spender.to_string(), None, None)
        .unwrap();
    assert_eq!(resp.allowances, vec![]);

    // set allowance with height expiration
    let allow1 = Uint128::new(7777);
    let expires = Expiration::AtHeight(123_456);
    contract
        .cw20_allowances_proxy()
        .increase_allowance(spender.to_string(), allow1, Some(expires))
        .call(owner)
        .unwrap();

    // set allowance with no expiration, from the other owner
    let contract2 = code_id
        .instantiate(InstantiateMsgData {
            name: "Auto Gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: owner.into(),
                amount: start_amount,
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner2)
        .unwrap();

    let allow2 = Uint128::new(54321);
    contract2
        .cw20_allowances_proxy()
        .increase_allowance(spender.to_string(), allow2, None)
        .call(owner2)
        .unwrap();

    // query list on both contracts
    let resp = contract
        .cw20_allowances_proxy()
        .all_spender_allowances(spender.to_string(), None, None)
        .unwrap();
    assert_eq!(resp.allowances.len(), 1);

    let resp = contract2
        .cw20_allowances_proxy()
        .all_spender_allowances(spender.to_string(), None, None)
        .unwrap();
    assert_eq!(resp.allowances.len(), 1);
}

#[test]
fn query_all_accounts_works() {
    let app = App::default();

    // insert order and lexicographical order are different
    let owner = "owner";
    let acct2 = "zebra";
    let acct3 = "nice";
    let acct4 = "aaaardvark";
    let start_amount = Uint128::new(12340000);
    let expected_order = [
        acct4.to_string(),
        acct3.to_string(),
        owner.to_string(),
        acct2.to_string(),
    ];

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
            name: "Auto Gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 3,
            initial_balances: vec![Cw20Coin {
                address: owner.into(),
                amount: start_amount,
            }],
            mint: None,
            marketing: None,
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    // put money everywhere (to create balanaces)
    contract
        .transfer(acct2.to_string(), Uint128::new(222222))
        .call(owner)
        .unwrap();
    contract
        .transfer(acct3.to_string(), Uint128::new(333333))
        .call(owner)
        .unwrap();
    contract
        .transfer(acct4.to_string(), Uint128::new(444444))
        .call(owner)
        .unwrap();

    // make sure we get the proper results
    let resp = contract
        .cw20_allowances_proxy()
        .all_accounts(None, None)
        .unwrap();
    assert_eq!(resp.accounts, expected_order);

    // let's do pagination
    let resp = contract
        .cw20_allowances_proxy()
        .all_accounts(None, Some(2))
        .unwrap();
    assert_eq!(resp.accounts, expected_order[0..2].to_vec());

    let resp = contract
        .cw20_allowances_proxy()
        .all_accounts(Some(resp.accounts[1].clone()), Some(1))
        .unwrap();
    assert_eq!(resp.accounts, expected_order[2..3].to_vec());

    let resp = contract
        .cw20_allowances_proxy()
        .all_accounts(Some(resp.accounts[0].clone()), Some(777))
        .unwrap();
    assert_eq!(resp.accounts, expected_order[3..].to_vec());
}

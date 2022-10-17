use cosmwasm_std::{coin, coins, Addr};
use cw2::{query_contract_info, ContractVersion};
use cw_multi_test::{next_block, App};
use cw_utils::{Expiration, NativeBalance};

use crate::contract::{CONTRACT_NAME, CONTRACT_VERSION};

use super::proxy::Cw1SubkeysCodeId;

const ATOM: &str = "atom";

/// Helper function for comparing vectors or another slice-like object as they would represent
/// set with duplications. Compares sets by first sorting elements using provided ordering.
/// This functions reshufless elements inplace, as it should never matter as compared
/// containers should represent same value regardless of ordering, and making this inplace just
/// safes obsolete copying.
///
/// This is implemented as a macro instead of function to throw panic in the place of macro
/// usage instead of from function called inside test.
macro_rules! assert_sorted_eq {
    ($left:expr, $right:expr, $cmp:expr $(,)?) => {
        let mut left = $left;
        left.sort_by(&$cmp);

        let mut right = $right;
        right.sort_by($cmp);

        assert_eq!(left, right);
    };
}

#[test]
fn get_contract_version_works() {
    let mut app = App::default();

    let owner = Addr::unchecked("owner");

    let code_id = Cw1SubkeysCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(&mut app, &owner, &[], true, "Sublist contract")
        .unwrap();

    let version: ContractVersion = query_contract_info(&app, contract.addr().clone()).unwrap();

    assert_eq!(
        ContractVersion {
            contract: CONTRACT_NAME.to_string(),
            version: CONTRACT_VERSION.to_string(),
        },
        version
    );
}

mod allowance {
    use crate::responses::AllowanceInfo;
    use crate::state::Allowance;

    use super::*;

    #[test]
    fn query() {
        let mut app = App::default();

        let owner = Addr::unchecked("owner");
        let spender1 = Addr::unchecked("spender1");
        let spender2 = Addr::unchecked("spender2");
        let spender3 = Addr::unchecked("spender3");

        let code_id = Cw1SubkeysCodeId::store_code(&mut app);

        let contract = code_id
            .instantiate(&mut app, &owner, &[&owner], false, "Sublist contract")
            .unwrap();

        contract
            .increase_allowance(&mut app, &owner, &spender1, coin(1, ATOM), None)
            .unwrap();

        contract
            .increase_allowance(&mut app, &owner, &spender2, coin(2, ATOM), None)
            .unwrap();

        assert_eq!(
            Allowance {
                balance: NativeBalance(coins(1, ATOM)),
                expires: Expiration::Never {},
            },
            contract.allowance(&app, &spender1).unwrap()
        );

        assert_eq!(
            Allowance {
                balance: NativeBalance(coins(2, ATOM)),
                expires: Expiration::Never {},
            },
            contract.allowance(&app, &spender2).unwrap()
        );

        assert_eq!(
            Allowance::default(),
            contract.allowance(&app, &spender3).unwrap()
        );
    }

    #[test]
    fn query_expired() {
        let mut app = App::default();

        let owner = Addr::unchecked("owner");
        let spender = Addr::unchecked("spender");

        let code_id = Cw1SubkeysCodeId::store_code(&mut app);

        let contract = code_id
            .instantiate(&mut app, &owner, &[&owner], false, "Sublist contract")
            .unwrap();

        let height = app.block_info().height;
        contract
            .increase_allowance(
                &mut app,
                &owner,
                &spender,
                coin(1, ATOM),
                Expiration::AtHeight(height + 1),
            )
            .unwrap();

        app.update_block(next_block);

        // Check allowances work for accounts with balances
        assert_eq!(
            Allowance {
                balance: NativeBalance(vec![]),
                expires: Expiration::Never {},
            },
            contract.allowance(&app, &spender).unwrap()
        );
    }

    #[test]
    fn query_all() {
        let mut app = App::default();

        let owner = Addr::unchecked("owner");
        let spender1 = Addr::unchecked("spender1");
        let spender2 = Addr::unchecked("spender2");
        let spender3 = Addr::unchecked("spender3");
        let spender4 = Addr::unchecked("spender4");

        let code_id = Cw1SubkeysCodeId::store_code(&mut app);

        let contract = code_id
            .instantiate(&mut app, &owner, &[&owner], false, "Sublist contract")
            .unwrap();

        let height = app.block_info().height;
        contract
            .increase_allowance(&mut app, &owner, &spender1, coin(1234, ATOM), None)
            .unwrap();

        contract
            .increase_allowance(
                &mut app,
                &owner,
                &spender2,
                coin(2345, ATOM),
                Expiration::Never {},
            )
            .unwrap();

        contract
            .increase_allowance(
                &mut app,
                &owner,
                &spender3,
                coin(3456, ATOM),
                Expiration::AtHeight(height + 2),
            )
            .unwrap();

        contract
            .increase_allowance(
                &mut app,
                &owner,
                &spender4,
                coin(2222, ATOM),
                Expiration::AtHeight(height + 1),
            )
            .unwrap();

        app.update_block(next_block);

        let batch1 = contract.all_allowances(&app, None, 2).unwrap().allowances;
        assert_eq!(2, batch1.len());

        let batch2 = contract
            .all_allowances(&app, &batch1.last().unwrap().spender, 2)
            .unwrap()
            .allowances;
        assert_eq!(1, batch2.len());

        let height = app.block_info().height;
        let expected = vec![
            AllowanceInfo {
                spender: spender1,
                balance: NativeBalance(coins(1234, ATOM)),
                expires: Expiration::Never {}, // Not set, expected default
            },
            AllowanceInfo {
                spender: spender2,
                balance: NativeBalance(coins(2345, ATOM)),
                expires: Expiration::Never {},
            },
            AllowanceInfo {
                spender: spender3,
                balance: NativeBalance(coins(3456, ATOM)),
                expires: Expiration::AtHeight(height + 1),
            },
        ];

        // Check allowances work for accounts with balances
        assert_sorted_eq!(
            expected,
            [batch1, batch2].concat(),
            AllowanceInfo::cmp_by_spender
        );
    }
}

mod permissions {
    use crate::responses::PermissionsInfo;
    use crate::state::Permissions;

    use super::*;

    const ALL_PERMS: Permissions = Permissions {
        delegate: true,
        redelegate: true,
        undelegate: true,
        withdraw: true,
    };

    const NO_PERMS: Permissions = Permissions {
        delegate: false,
        redelegate: false,
        undelegate: false,
        withdraw: false,
    };

    #[test]
    fn query() {
        let mut app = App::default();

        let owner = Addr::unchecked("owner");
        let spender1 = Addr::unchecked("spender1");
        let spender2 = Addr::unchecked("spender2");
        let spender3 = Addr::unchecked("spender2");

        let code_id = Cw1SubkeysCodeId::store_code(&mut app);

        let contract = code_id
            .instantiate(&mut app, &owner, &[&owner], false, "Subkeys contract")
            .unwrap();

        contract
            .set_permission(&mut app, &owner, &spender1, ALL_PERMS)
            .unwrap();

        contract
            .set_permission(&mut app, &owner, &spender2, NO_PERMS)
            .unwrap();

        assert_eq!(ALL_PERMS, contract.permissions(&app, &spender1).unwrap());
        assert_eq!(NO_PERMS, contract.permissions(&app, &spender2).unwrap());
        assert_eq!(NO_PERMS, contract.permissions(&app, &spender3).unwrap());
    }

    #[test]
    fn query_all() {
        let mut app = App::default();

        let owner = Addr::unchecked("owner");
        let spender1 = Addr::unchecked("spender1");
        let spender2 = Addr::unchecked("spender2");
        let spender3 = Addr::unchecked("spender3");

        let code_id = Cw1SubkeysCodeId::store_code(&mut app);

        let contract = code_id
            .instantiate(&mut app, &owner, &[&owner], false, "Subkeys contract")
            .unwrap();

        contract
            .set_permission(&mut app, &owner, &spender1, ALL_PERMS)
            .unwrap();

        contract
            .set_permission(&mut app, &owner, &spender2, NO_PERMS)
            .unwrap();

        contract
            .set_permission(&mut app, &owner, &spender3, NO_PERMS)
            .unwrap();

        assert_eq!(ALL_PERMS, contract.permissions(&app, &spender1).unwrap());
        assert_eq!(NO_PERMS, contract.permissions(&app, &spender2).unwrap());
        assert_eq!(NO_PERMS, contract.permissions(&app, &spender3).unwrap());

        let batch1 = contract.all_permissions(&app, None, 2).unwrap().permissions;
        assert_eq!(2, batch1.len());

        let batch2 = contract
            .all_permissions(&app, &batch1.last().unwrap().spender, 2)
            .unwrap()
            .permissions;
        assert_eq!(1, batch2.len());

        let expected = vec![
            PermissionsInfo {
                spender: spender1,
                permissions: ALL_PERMS,
            },
            PermissionsInfo {
                spender: spender2,
                permissions: NO_PERMS,
            },
            PermissionsInfo {
                spender: spender3,
                permissions: NO_PERMS,
            },
        ];

        // Check allowances work for accounts with balances
        assert_sorted_eq!(
            expected,
            [batch1, batch2].concat(),
            PermissionsInfo::cmp_by_spender
        );
    }
}

mod cw1_execute {
    use cosmwasm_std::BankMsg;

    use super::*;

    #[test]
    fn can_execute() {
        let mut app = App::default();

        let owner = Addr::unchecked("owner");
        let admin = Addr::unchecked("admin");
        let non_admin = Addr::unchecked("non_admin");

        let code_id = Cw1SubkeysCodeId::store_code(&mut app);

        let contract = code_id
            .instantiate(
                &mut app,
                &owner,
                &[&owner, &admin],
                false,
                "Subkeys contract",
            )
            .unwrap();

        let msg = BankMsg::Send {
            to_address: "owner".to_owned(),
            amount: vec![],
        };

        let resp = contract
            .can_execute(&app, admin.to_string(), msg.clone().into())
            .unwrap();

        assert!(resp.can_execute);

        let resp = contract
            .can_execute(&app, non_admin.to_string(), msg.into())
            .unwrap();

        assert!(!resp.can_execute);
    }

    #[test]
    fn execute() {
        let owner = Addr::unchecked("owner");
        let admin = Addr::unchecked("admin");
        let non_admin = Addr::unchecked("non_admin");

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &admin, coins(2345, ATOM))
                .unwrap();
            router
                .bank
                .init_balance(storage, &non_admin, coins(2345, ATOM))
                .unwrap();
        });

        let code_id = Cw1SubkeysCodeId::store_code(&mut app);

        let contract = code_id
            .instantiate(
                &mut app,
                &owner,
                &[&owner, &admin],
                false,
                "Subkeys contract",
            )
            .unwrap();

        let msg = BankMsg::Send {
            to_address: "owner".to_owned(),
            amount: vec![coin(2345, ATOM)],
        };

        contract
            .execute(
                &mut app,
                admin,
                vec![msg.clone().into()],
                &[coin(2345, ATOM)],
            )
            .unwrap();

        contract
            .execute(&mut app, non_admin, vec![msg.into()], &[coin(2345, ATOM)])
            .unwrap_err();
    }
}

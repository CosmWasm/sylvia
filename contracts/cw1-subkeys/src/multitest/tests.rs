use cosmwasm_std::{coin, coins, Addr};
use cw2::{query_contract_info, ContractVersion};
use cw_utils::{Expiration, NativeBalance};
use sylvia::multitest::App;

use crate::contract::multitest_utils::CodeId;
use crate::contract::{CONTRACT_NAME, CONTRACT_VERSION};

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
    let app = App::default();

    let owner = "owner";

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(vec![owner.to_owned()], true)
        .with_label("Sublist contract")
        .call(owner)
        .unwrap();

    let version: ContractVersion =
        query_contract_info(&app.app().wrap(), contract.contract_addr.to_string()).unwrap();

    assert_eq!(
        ContractVersion {
            contract: CONTRACT_NAME.to_string(),
            version: CONTRACT_VERSION.to_string(),
        },
        version
    );
}

mod allowance {
    use cw_multi_test::next_block;

    use crate::responses::AllowanceInfo;
    use crate::state::Allowance;

    use super::*;

    #[test]
    fn query() {
        let app = App::default();

        let owner = "owner";
        let spenders = ["spender1", "spender2", "spender3"];

        let code_id = CodeId::store_code(&app);

        let contract = code_id
            .instantiate(vec![owner.to_owned()], false)
            .with_label("Sublist contract")
            .call(owner)
            .unwrap();

        contract
            .increase_allowance(spenders[0].to_owned(), coin(1, ATOM), None)
            .call(owner)
            .unwrap();

        contract
            .increase_allowance(spenders[1].to_owned(), coin(2, ATOM), None)
            .call(owner)
            .unwrap();

        assert_eq!(
            Allowance {
                balance: NativeBalance(coins(1, ATOM)),
                expires: Expiration::Never {},
            },
            contract.allowance(spenders[0].to_owned()).unwrap()
        );

        assert_eq!(
            Allowance {
                balance: NativeBalance(coins(2, ATOM)),
                expires: Expiration::Never {},
            },
            contract.allowance(spenders[1].to_owned()).unwrap()
        );

        assert_eq!(
            Allowance::default(),
            contract.allowance(spenders[2].to_owned()).unwrap()
        );
    }

    #[test]
    fn query_expired() {
        let app = App::default();

        let owner = "owner";
        let spender = "spender";

        let code_id = CodeId::store_code(&app);

        let contract = code_id
            .instantiate(vec![owner.to_owned()], false)
            .with_label("Sublist contract")
            .call(owner)
            .unwrap();

        let height = app.block_info().height;
        contract
            .increase_allowance(
                spender.to_owned(),
                coin(1, ATOM),
                Some(Expiration::AtHeight(height + 1)),
            )
            .call(owner)
            .unwrap();

        app.update_block(next_block);

        // Check allowances work for accounts with balances
        assert_eq!(
            Allowance {
                balance: NativeBalance(vec![]),
                expires: Expiration::Never {},
            },
            contract.allowance(spender.to_owned()).unwrap()
        );
    }

    #[test]
    fn query_all() {
        let app = App::default();

        let owner = "owner";
        let spender1 = "spender1";
        let spender2 = "spender2";
        let spender3 = "spender3";
        let spender4 = "spender4";

        let code_id = CodeId::store_code(&app);

        let contract = code_id
            .instantiate(vec![owner.to_owned()], false)
            .with_label("Sublist contract")
            .call(owner)
            .unwrap();

        let height = app.block_info().height;
        contract
            .increase_allowance(spender1.to_owned(), coin(1234, ATOM), None)
            .call(owner)
            .unwrap();

        contract
            .increase_allowance(
                spender2.to_owned(),
                coin(2345, ATOM),
                Some(Expiration::Never {}),
            )
            .call(owner)
            .unwrap();

        contract
            .increase_allowance(
                spender3.to_owned(),
                coin(3456, ATOM),
                Some(Expiration::AtHeight(height + 2)),
            )
            .call(owner)
            .unwrap();

        contract
            .increase_allowance(
                spender4.to_owned(),
                coin(2222, ATOM),
                Some(Expiration::AtHeight(height + 1)),
            )
            .call(owner)
            .unwrap();

        app.update_block(next_block);

        let batch1 = contract.all_allowances(None, Some(2)).unwrap().allowances;
        assert_eq!(2, batch1.len());

        let batch2 = contract
            .all_allowances(Some(batch1.last().unwrap().spender.to_string()), Some(2))
            .unwrap()
            .allowances;
        assert_eq!(1, batch2.len());

        let height = app.block_info().height;
        let expected = vec![
            AllowanceInfo {
                spender: Addr::unchecked(spender1),
                balance: NativeBalance(coins(1234, ATOM)),
                expires: Expiration::Never {}, // Not set, expected default
            },
            AllowanceInfo {
                spender: Addr::unchecked(spender2),
                balance: NativeBalance(coins(2345, ATOM)),
                expires: Expiration::Never {},
            },
            AllowanceInfo {
                spender: Addr::unchecked(spender3),
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
        let app = App::default();

        let owner = "owner";
        let spender1 = "spender1";
        let spender2 = "spender2";
        let spender3 = "spender2";

        let code_id = CodeId::store_code(&app);

        let contract = code_id
            .instantiate(vec![owner.to_string()], false)
            .with_label("Subkeys contract")
            .call(owner)
            .unwrap();

        contract
            .set_permissions(spender1.to_string(), ALL_PERMS)
            .call(owner)
            .unwrap();

        contract
            .set_permissions(spender2.to_string(), NO_PERMS)
            .call(owner)
            .unwrap();

        assert_eq!(
            ALL_PERMS,
            contract.permissions(spender1.to_string()).unwrap()
        );
        assert_eq!(
            NO_PERMS,
            contract.permissions(spender2.to_string()).unwrap()
        );
        assert_eq!(
            NO_PERMS,
            contract.permissions(spender3.to_string()).unwrap()
        );
    }

    #[test]
    fn query_all() {
        let app = App::default();

        let owner = "owner";
        let spender1 = "spender1";
        let spender2 = "spender2";
        let spender3 = "spender3";

        let code_id = CodeId::store_code(&app);

        let contract = code_id
            .instantiate(vec![owner.to_owned()], false)
            .with_label("Subkeys contract")
            .call(owner)
            .unwrap();

        contract
            .set_permissions(spender1.to_owned(), ALL_PERMS)
            .call(owner)
            .unwrap();

        contract
            .set_permissions(spender2.to_owned(), NO_PERMS)
            .call(owner)
            .unwrap();

        contract
            .set_permissions(spender3.to_owned(), NO_PERMS)
            .call(owner)
            .unwrap();

        assert_eq!(
            ALL_PERMS,
            contract.permissions(spender1.to_owned()).unwrap()
        );
        assert_eq!(NO_PERMS, contract.permissions(spender2.to_owned()).unwrap());
        assert_eq!(NO_PERMS, contract.permissions(spender3.to_owned()).unwrap());

        let batch1 = contract.all_permissions(None, Some(2)).unwrap().permissions;
        assert_eq!(2, batch1.len());

        let batch2 = contract
            .all_permissions(Some(batch1.last().unwrap().spender.to_string()), Some(2))
            .unwrap()
            .permissions;
        assert_eq!(1, batch2.len());

        let expected = vec![
            PermissionsInfo {
                spender: Addr::unchecked(spender1),
                permissions: ALL_PERMS,
            },
            PermissionsInfo {
                spender: Addr::unchecked(spender2),
                permissions: NO_PERMS,
            },
            PermissionsInfo {
                spender: Addr::unchecked(spender3),
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
    use crate::cw1::test_utils::Cw1Methods;
    use cosmwasm_std::BankMsg;

    use super::*;

    #[test]
    fn can_execute() {
        let app = App::default();

        let owner = "owner";
        let admin = "admin";
        let non_admin = "non_admin";

        let code_id = CodeId::store_code(&app);

        let contract = code_id
            .instantiate(vec![owner.to_owned(), admin.to_owned()], false)
            .with_label("Subkeys contract")
            .call(owner)
            .unwrap();

        let msg = BankMsg::Send {
            to_address: "owner".to_owned(),
            amount: vec![],
        };

        let resp = contract
            .cw1_proxy()
            .can_execute(admin.to_string(), msg.clone().into())
            .unwrap();

        assert!(resp.can_execute);

        let resp = contract
            .cw1_proxy()
            .can_execute(non_admin.to_string(), msg.into())
            .unwrap();

        assert!(!resp.can_execute);
    }

    #[test]
    fn execute() {
        let owner = "owner";
        let admin = "admin";
        let non_admin = "non_admin";

        let app = cw_multi_test::App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(admin), coins(2345, ATOM))
                .unwrap();
            router
                .bank
                .init_balance(storage, &Addr::unchecked(non_admin), coins(2345, ATOM))
                .unwrap();
        });

        let app = App::new(app);

        let code_id = CodeId::store_code(&app);

        let contract = code_id
            .instantiate(vec![owner.to_owned(), admin.to_string()], false)
            .with_label("Subkeys contract")
            .call(owner)
            .unwrap();

        let msg = BankMsg::Send {
            to_address: "owner".to_owned(),
            amount: vec![coin(2345, ATOM)],
        };

        contract
            .cw1_proxy()
            .execute(vec![msg.clone().into()])
            .with_funds(&[coin(2345, ATOM)])
            .call(admin)
            .unwrap();

        contract
            .cw1_proxy()
            .execute(vec![msg.into()])
            .with_funds(&[coin(2345, ATOM)])
            .call(non_admin)
            .unwrap_err();
    }
}

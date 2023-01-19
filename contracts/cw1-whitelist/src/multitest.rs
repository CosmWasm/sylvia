#[cfg(test)]
mod test {
    use cosmwasm_std::{to_binary, WasmMsg};

    use crate::contract::multitest_utils::Cw1WhitelistContractCodeId;
    use crate::error::ContractError;
    use crate::responses::AdminListResponse;
    use crate::whitelist;
    use assert_matches::assert_matches;

    #[test]
    fn proxy_freeze_message() {
        let mut app = sylvia::multitest::App::default();
        let code_id = Cw1WhitelistContractCodeId::store_code(&mut app);

        let owner = "owner";

        let first_contract = code_id
            .instantiate()
            .with_label("First contract")
            .call(owner, vec![owner.to_owned()], true)
            .unwrap();

        let second_contract = code_id
            .instantiate()
            .with_label("Second contract")
            .call(owner, vec![first_contract.contract_addr.to_string()], true)
            .unwrap();

        let freeze = whitelist::ExecMsg::Freeze {};
        let freeze = WasmMsg::Execute {
            contract_addr: second_contract.contract_addr.to_string(),
            msg: to_binary(&freeze).unwrap(),
            funds: vec![],
        };

        first_contract
            .cw1_proxy()
            .execute(vec![freeze.into()])
            .with_sender(owner)
            .call()
            .unwrap();

        let resp = second_contract.whitelist_proxy().admin_list().unwrap();

        assert_matches!(
            resp,
            AdminListResponse {
                mutable,
                ..
            } if !mutable
        );
    }

    #[test]
    fn update_admins() {
        let mut app = sylvia::multitest::App::default();
        let code_id = Cw1WhitelistContractCodeId::store_code(&mut app);

        let owner = "owner";
        let mut admins = vec!["admin1".to_owned(), "admin2".to_owned()];

        let contract = code_id
            .instantiate()
            .call(owner, admins.clone(), true)
            .unwrap();

        let resp = contract.whitelist_proxy().admin_list().unwrap();
        assert_eq!(resp.admins, admins);

        admins.push("admin3".to_owned());
        contract
            .whitelist_proxy()
            .update_admins(admins.clone())
            .with_sender("admin1")
            .call()
            .unwrap();

        let resp = contract.whitelist_proxy().admin_list().unwrap();
        assert_eq!(resp.admins, admins);
    }

    #[test]
    fn unathorized_admin_update() {
        let mut app = sylvia::multitest::App::default();
        let code_id = Cw1WhitelistContractCodeId::store_code(&mut app);

        let owner = "owner";

        let contract = code_id
            .instantiate()
            .call(owner, vec![owner.to_string()], true)
            .unwrap();

        let err = contract
            .whitelist_proxy()
            .update_admins(vec![owner.to_owned(), "fake_admin".to_owned()])
            .with_sender("fake_admin")
            .call()
            .unwrap_err();

        assert_eq!(err, ContractError::Unauthorized {});

        contract
            .whitelist_proxy()
            .freeze()
            .with_sender(owner)
            .call()
            .unwrap();

        let err = contract
            .whitelist_proxy()
            .update_admins(vec![owner.to_owned(), "admin".to_owned()])
            .with_sender(owner)
            .call()
            .unwrap_err();

        assert_eq!(err, ContractError::ContractFrozen {});
    }
}

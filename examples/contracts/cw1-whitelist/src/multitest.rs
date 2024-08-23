#[cfg(test)]
mod test {
    use cw_multi_test::IntoBech32;
    use sylvia::cw_std::{to_json_binary, Addr, WasmMsg};
    use whitelist::responses::AdminListResponse;

    use crate::contract::sv::mt::CodeId;
    use crate::error::ContractError;
    use assert_matches::assert_matches;
    use sylvia::multitest::App;

    use cw1::sv::mt::Cw1Proxy;
    use whitelist::sv::mt::WhitelistProxy;

    #[test]
    fn proxy_freeze_message() {
        let app = App::default();
        let code_id = CodeId::store_code(&app);

        let owner = "owner".into_bech32();

        let first_contract = code_id
            .instantiate(vec![owner.to_string()], true)
            .with_label("First contract")
            .call(&owner)
            .unwrap();

        let second_contract = code_id
            .instantiate(vec![first_contract.contract_addr.to_string()], true)
            .with_label("Second contract")
            .call(&owner)
            .unwrap();

        let freeze = whitelist::sv::ExecMsg::Freeze {};
        let freeze = WasmMsg::Execute {
            contract_addr: second_contract.contract_addr.to_string(),
            msg: to_json_binary(&freeze).unwrap(),
            funds: vec![],
        };

        first_contract
            .execute(vec![freeze.into()])
            .call(&owner)
            .unwrap();

        let resp = second_contract.admin_list().unwrap();

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
        let app = App::default();
        let code_id = CodeId::store_code(&app);

        let owner = "owner".into_bech32();
        let admin1 = "admin1".into_bech32();
        let admin2 = "admin2".into_bech32();
        let admin3 = "admin3".into_bech32();
        let mut admins = vec![admin1.to_string(), admin2.to_string()];
        admins.sort();

        let contract = code_id
            .instantiate(admins.clone(), true)
            .call(&owner)
            .unwrap();

        let resp = contract.admin_list().unwrap();
        assert_eq!(resp.admins, admins);

        admins.push(admin3.to_string());
        admins.sort();
        contract
            .update_admins(admins.clone())
            .call(&admin1)
            .unwrap();

        let resp = contract.admin_list().unwrap();
        assert_eq!(resp.admins, admins);
    }

    #[test]
    fn unathorized_admin_update() {
        let app = App::default();
        let code_id = CodeId::store_code(&app);

        let owner = "owner".into_bech32();

        let contract = code_id
            .instantiate(vec![owner.to_string()], true)
            .call(&owner)
            .unwrap();

        let err = contract
            .update_admins(vec![owner.to_string(), "fake_admin".to_owned()])
            .call(&Addr::unchecked("fake_admin"))
            .unwrap_err();

        assert_eq!(err, ContractError::Unauthorized);

        contract.freeze().call(&owner).unwrap();

        let err = contract
            .update_admins(vec![owner.to_string(), "admin".to_owned()])
            .call(&owner)
            .unwrap_err();

        assert_eq!(err, ContractError::ContractFrozen);
    }
}

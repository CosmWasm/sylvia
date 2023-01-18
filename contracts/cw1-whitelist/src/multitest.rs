#[cfg(test)]
mod test {
    use cosmwasm_std::{to_binary, Addr, WasmMsg};
    use cw_multi_test::{App, Executor};

    use crate::contract::multitest_utils::Cw1WhitelistContractCodeId;
    use crate::contract::{Cw1WhitelistContract, InstantiateMsg};
    use crate::responses::AdminListResponse;
    use crate::whitelist;
    use assert_matches::assert_matches;

    #[test]
    fn proxy_freeze_message() {
        let mut app = App::default();

        let owner = Addr::unchecked("owner");

        let code_id = app.store_code(Box::new(Cw1WhitelistContract::new()));

        let first_contract = app
            .instantiate_contract(
                code_id,
                owner.clone(),
                &InstantiateMsg {
                    admins: vec![owner.to_string()],
                    mutable: true,
                },
                &[],
                "First contract",
                None,
            )
            .unwrap();

        let second_contract = app
            .instantiate_contract(
                code_id,
                owner.clone(),
                &InstantiateMsg {
                    admins: vec![first_contract.to_string()],
                    mutable: true,
                },
                &[],
                "Second contract",
                None,
            )
            .unwrap();
        assert_ne!(second_contract, first_contract);

        let freeze = whitelist::ExecMsg::Freeze {};
        let freeze = WasmMsg::Execute {
            contract_addr: second_contract.to_string(),
            msg: to_binary(&freeze).unwrap(),
            funds: vec![],
        };
        app.execute_contract(
            owner,
            first_contract,
            &cw1::ExecMsg::Execute {
                msgs: vec![freeze.into()],
            },
            &[],
        )
        .unwrap();

        let resp = app
            .wrap()
            .query_wasm_smart(second_contract, &whitelist::QueryMsg::AdminList {})
            .unwrap();

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
}

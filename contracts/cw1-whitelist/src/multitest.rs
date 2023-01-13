#[cfg(test)]
mod test {
    use cosmwasm_std::{to_binary, Addr, WasmMsg};
    use cw_multi_test::{App, Executor};

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
}

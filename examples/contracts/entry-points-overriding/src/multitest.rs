#[cfg(test)]
mod test {
    use crate::contract::sv::multitest_utils::CodeId;
    use crate::contract::sv::{ContractExecMsg, ExecMsg};
    use crate::messages::{CustomExecMsg, SudoMsg, UserExecMsg};
    use cosmwasm_std::Addr;
    use cw_multi_test::Executor;
    use sylvia::multitest::App;

    #[test]
    fn overriden_entry_points_in_mt() {
        let app = App::default();
        let code_id = CodeId::store_code(&app);

        let owner = "owner";

        let contract = code_id
            .instantiate()
            .with_label("Contract")
            .with_admin(Some(owner))
            .call(owner)
            .unwrap();

        let count = contract.count().unwrap().count;
        assert_eq!(count, 0);

        let msg = SudoMsg::SetCountToThree {};

        contract
            .app
            .app_mut()
            .wasm_sudo(contract.contract_addr.clone(), &msg)
            .unwrap();

        let count = contract.count().unwrap().count;
        assert_eq!(count, 3);

        // custom ExecMsg
        let msg = CustomExecMsg::CustomExec(UserExecMsg::IncreaseByOne {});
        (*contract.app)
            .app_mut()
            .execute_contract(
                Addr::unchecked(owner),
                contract.contract_addr.clone(),
                &msg,
                &[],
            )
            .unwrap();

        let count = contract.count().unwrap().count;
        assert_eq!(count, 4);

        // custom ExecMsg
        let msg = CustomExecMsg::ContractExec(ContractExecMsg::CounterContract(
            ExecMsg::increase_by_two(),
        ));
        (*contract.app)
            .app_mut()
            .execute_contract(
                Addr::unchecked(owner),
                contract.contract_addr.clone(),
                &msg,
                &[],
            )
            .unwrap();

        let count = contract.count().unwrap().count;
        assert_eq!(count, 6);
    }
}

#[cfg(test)]
mod test {
    use crate::contract::{Cw1WhitelistContract, InstantiateMsg};
    use anyhow::{bail, Result as AnyResult};
    use cosmwasm_std::{
        from_slice, Addr, Binary, DepsMut, Empty, Env, MessageInfo, Reply, Response,
    };
    use cw1::{ExecMsg, FindMemberResponse, QueryMsg};
    use cw_multi_test::{AppBuilder, Contract, Executor};

    impl Contract<Empty> for Cw1WhitelistContract {
        fn instantiate(
            &self,
            deps: DepsMut,
            env: Env,
            info: MessageInfo,
            msg: Vec<u8>,
        ) -> AnyResult<Response<Empty>> {
            from_slice::<InstantiateMsg>(&msg)?
                .dispatch(self, (deps, env, info))
                .map_err(Into::into)
        }

        fn execute(
            &self,
            deps: DepsMut,
            env: Env,
            info: MessageInfo,
            msg: Vec<u8>,
        ) -> AnyResult<Response<Empty>> {
            from_slice::<ExecMsg>(&msg)?
                .dispatch(self, (deps, env, info))
                .map_err(Into::into)
        }

        fn query(&self, deps: cosmwasm_std::Deps, env: Env, msg: Vec<u8>) -> AnyResult<Binary> {
            from_slice::<QueryMsg>(&msg)?
                .dispatch(self, (deps, env))
                .map_err(Into::into)
        }

        fn sudo(&self, _deps: DepsMut, _env: Env, _msg: Vec<u8>) -> AnyResult<Response<Empty>> {
            bail!("sudo not implemented for contract")
        }

        fn reply(&self, _deps: DepsMut, _env: Env, _msg: Reply) -> AnyResult<Response<Empty>> {
            bail!("reply not implemented for contract")
        }

        fn migrate(&self, _deps: DepsMut, _env: Env, _msg: Vec<u8>) -> AnyResult<Response<Empty>> {
            bail!("migrate not implemented for contract")
        }
    }

    #[test]
    fn entry_points() {
        let mut app = AppBuilder::new().build(|_, _, _| ());
        let contract_id = app.store_code(Box::new(Cw1WhitelistContract::new()));

        let contract = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("Owner"),
                &InstantiateMsg {
                    members: vec!["member".to_owned()],
                },
                &[],
                "contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("owner"),
            contract.clone(),
            &ExecMsg::AddMember {
                member: "other_member".to_owned(),
            },
            &[],
        )
        .unwrap();

        let resp: FindMemberResponse = app
            .wrap()
            .query_wasm_smart(
                contract,
                &QueryMsg::FindMember {
                    member: "other_member".to_owned(),
                },
            )
            .unwrap();

        assert_eq!(resp, FindMemberResponse { is_present: true });
    }
}

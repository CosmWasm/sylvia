use anyhow::{bail, Result as AnyResult};
use cosmwasm_std::{from_slice, Binary, DepsMut, Empty, Env, MessageInfo, Reply, Response};
use cw_multi_test::Contract;

use crate::contract::{ContractExecMsg, ContractQueryMsg, Cw1WhitelistContract, InstantiateMsg};

impl Contract<Empty> for Cw1WhitelistContract<'_> {
    fn instantiate(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Vec<u8>,
    ) -> AnyResult<Response> {
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
        from_slice::<ContractExecMsg>(&msg)?
            .dispatch(self, (deps, env, info))
            .map_err(Into::into)
    }

    fn query(&self, deps: cosmwasm_std::Deps, env: Env, msg: Vec<u8>) -> AnyResult<Binary> {
        from_slice::<ContractQueryMsg>(&msg)?
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

#[cfg(test)]
mod test {
    use cosmwasm_std::{to_binary, Addr, WasmMsg};
    use cw_multi_test::{App, Executor};

    use crate::contract::{ImplExecMsg, ImplQueryMsg};
    use crate::responses::AdminListResponse;
    use assert_matches::assert_matches;

    use super::*;

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

        let freeze = ImplExecMsg::Freeze {};
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
            .query_wasm_smart(second_contract, &ImplQueryMsg::AdminList {})
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

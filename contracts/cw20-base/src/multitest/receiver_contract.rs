use anyhow::{bail, Result as AnyResult};
use cosmwasm_std::{
    from_slice, Addr, DepsMut, Empty, Env, MessageInfo, Response, StdError, StdResult,
};
use cw_multi_test::{App, Contract, Executor};
use sylvia::contract;
use thiserror::Error;

use super::receiver::{self, Receiver};

pub struct ReceiverContract {}

#[contract]
#[messages(receiver as Receiver)]
impl ReceiverContract {
    pub const fn new() -> Self {
        Self {}
    }
    #[msg(instantiate)]
    pub fn instantiate(&self, _ctx: (DepsMut, Env, MessageInfo)) -> StdResult<Response> {
        Ok(Response::new())
    }
}

impl Receiver for ReceiverContract {
    type Error = StdError;

    fn receive(
        &self,
        _ctx: (DepsMut, Env, MessageInfo),
        _sender: String,
        _amount: cosmwasm_std::Uint128,
        _msg: cosmwasm_std::Binary,
    ) -> Result<Response, Self::Error> {
        Ok(Response::default())
    }
}

impl Contract<Empty> for ReceiverContract {
    fn execute(
        &self,
        deps: cosmwasm_std::DepsMut<Empty>,
        env: cosmwasm_std::Env,
        info: cosmwasm_std::MessageInfo,
        msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        from_slice::<ContractExecMsg>(&msg)?
            .dispatch(self, (deps, env, info))
            .map_err(Into::into)
    }

    fn instantiate(
        &self,
        deps: cosmwasm_std::DepsMut<Empty>,
        env: cosmwasm_std::Env,
        info: cosmwasm_std::MessageInfo,
        msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        from_slice::<InstantiateMsg>(&msg)?
            .dispatch(self, (deps, env, info))
            .map_err(Into::into)
    }

    fn query(
        &self,
        deps: cosmwasm_std::Deps<Empty>,
        env: cosmwasm_std::Env,
        msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Binary> {
        from_slice::<ContractQueryMsg>(&msg)?
            .dispatch(self, (deps, env))
            .map_err(Into::into)
    }

    fn sudo(
        &self,
        _deps: cosmwasm_std::DepsMut<Empty>,
        _env: cosmwasm_std::Env,
        _msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        bail!("sudo not implemented for contract")
    }

    fn reply(
        &self,
        _deps: cosmwasm_std::DepsMut<Empty>,
        _env: cosmwasm_std::Env,
        _msg: cosmwasm_std::Reply,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        bail!("reply not implemented for contract")
    }

    fn migrate(
        &self,
        _deps: cosmwasm_std::DepsMut<Empty>,
        _env: cosmwasm_std::Env,
        _msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        bail!("migrate not implemented for contract")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReceiverContractCodeId(u64);

impl ReceiverContractCodeId {
    pub fn store_code(app: &mut App) -> Self {
        let code_id = app.store_code(Box::new(ReceiverContract::new()));
        Self(code_id)
    }

    #[track_caller]
    pub fn instantiate(
        self,
        app: &mut App,
        sender: &Addr,
        label: &str,
    ) -> StdResult<ReceiverContractProxy> {
        let msg = InstantiateMsg {};

        app.instantiate_contract(self.0, sender.clone(), &msg, &[], label, None)
            .map_err(|err| err.downcast().unwrap())
            .map(ReceiverContractProxy)
    }
}

pub struct ReceiverContractProxy(Addr);

impl ReceiverContractProxy {
    pub fn addr(&self) -> &Addr {
        &self.0
    }
}

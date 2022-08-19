use anyhow::{bail, Result as AnyResult};
use cosmwasm_std::{Binary, DepsMut, Empty, Env, MessageInfo, Reply, Response};
use cw_multi_test::Contract;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;

use crate::contract::Cw1WhitelistContract;

impl Contract<Empty> for Cw1WhitelistContract {
    fn instantiate(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Vec<u8>,
    ) -> AnyResult<Response<Empty>> {
        self.entry_instantiate(deps, env, info, &msg)
            .map_err(Into::into)
    }

    fn execute(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Vec<u8>,
    ) -> AnyResult<Response<Empty>> {
        self.entry_execute(deps, env, info, &msg)
            .map_err(Into::into)
    }

    fn query(&self, deps: cosmwasm_std::Deps, env: Env, msg: Vec<u8>) -> AnyResult<Binary> {
        self.entry_query(deps, env, &msg).map_err(Into::into)
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

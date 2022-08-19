mod contract;
mod multitest;

#[cfg(not(feature = "library"))]
pub mod entry_points {
    use anyhow::{bail, Error, Result as AnyResult};
    use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response};

    use crate::contract::Cw1WhitelistContract;

    const CONTRACT: Cw1WhitelistContract = Cw1WhitelistContract::new();

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Binary,
    ) -> Result<Response, Error> {
        CONTRACT.entry_instantiate(deps, env, info, &msg)
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Binary,
    ) -> anyhow::Result<Response, anyhow::Error> {
        CONTRACT.entry_execute(deps, env, info, &msg)
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: Binary) -> Result<Binary, Error> {
        CONTRACT.entry_query(deps, env, &msg)
    }

    #[entry_point]
    pub fn sudo(_deps: DepsMut, _env: Env, _msg: Vec<u8>) -> AnyResult<Response> {
        bail!("sudo not implemented for contract")
    }

    #[entry_point]
    pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> AnyResult<Response> {
        bail!("reply not implemented for contract")
    }

    #[entry_point]
    pub fn migrate(_deps: DepsMut, _env: Env, _msg: Vec<u8>) -> AnyResult<Response> {
        bail!("migrate not implemented for contract")
    }
}

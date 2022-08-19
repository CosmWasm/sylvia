mod contract;
mod multitest;

#[cfg(not(feature = "library"))]
pub mod entry_points {
    use anyhow::Error;
    use cosmwasm_std::{
        entry_point, from_slice, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    };

    use crate::contract::{
        contract::{ExecMsg, InstantiateMsg, QueryMsg},
        Cw1WhitelistContract,
    };

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
}

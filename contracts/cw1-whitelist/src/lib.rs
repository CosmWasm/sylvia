mod contract;

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
        let msg: InstantiateMsg = from_slice(&msg)?;
        msg.dispatch(&CONTRACT, (deps, env, info))
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Binary,
    ) -> Result<Response, Error> {
        let msg: ExecMsg = from_slice(&msg)?;
        msg.dispatch(&CONTRACT, (deps, env, info))
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: Binary) -> Result<Binary, Error> {
        let msg: QueryMsg = from_slice(&msg)?;
        msg.dispatch(&CONTRACT, (deps, env))
    }
}

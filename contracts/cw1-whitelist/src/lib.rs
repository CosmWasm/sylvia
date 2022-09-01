mod contract;
mod error;
mod multitest;

#[cfg(not(feature = "library"))]
pub mod entry_points {
    use cosmwasm_std::{
        entry_point, from_slice, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    };

    use crate::contract::{Cw1WhitelistContract, ExecMsg, InstantiateMsg, QueryMsg};
    use crate::error::ContractError;

    const CONTRACT: Cw1WhitelistContract = Cw1WhitelistContract::new();

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Binary,
    ) -> Result<Response, ContractError> {
        from_slice::<InstantiateMsg>(&msg)?.dispatch(&CONTRACT, (deps, env, info))
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Binary,
    ) -> Result<Response, ContractError> {
        from_slice::<ExecMsg>(&msg)?.dispatch(&CONTRACT, (deps, env, info))
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: Binary) -> Result<Binary, ContractError> {
        from_slice::<QueryMsg>(&msg)?.dispatch(&CONTRACT, (deps, env))
    }
}

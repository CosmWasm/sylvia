mod contract;
mod error;
mod multitest;

#[cfg(not(feature = "library"))]
pub mod entry_points {
    use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response};

    use crate::{contract::Cw1WhitelistContract, error::ContractError};

    const CONTRACT: Cw1WhitelistContract = Cw1WhitelistContract::new();

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Binary,
    ) -> Result<Response, ContractError> {
        CONTRACT.entry_instantiate(deps, env, info, &msg)
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Binary,
    ) -> Result<Response, ContractError> {
        CONTRACT.entry_execute(deps, env, info, &msg)
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: Binary) -> Result<Binary, ContractError> {
        CONTRACT.entry_query(deps, env, &msg)
    }
}

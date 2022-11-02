mod allowances;
mod contract;
mod error;
mod state;
mod validation;

#[cfg(any(test, feature = "tests"))]
pub mod multitest;

#[cfg(not(feature = "library"))]
mod entry_points {
    use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response};

    use crate::contract::{ContractExecMsg, ContractQueryMsg, Cw20Base, InstantiateMsg};
    use crate::error::ContractError;

    const CONTRACT: Cw20Base = Cw20Base::new();

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        msg.dispatch(&CONTRACT, (deps, env, info))
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ContractExecMsg,
    ) -> Result<Response, ContractError> {
        msg.dispatch(&CONTRACT, (deps, env, info))
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: ContractQueryMsg) -> Result<Binary, ContractError> {
        msg.dispatch(&CONTRACT, (deps, env))
    }
}

#[cfg(not(feature = "library"))]
pub use crate::entry_points::*;

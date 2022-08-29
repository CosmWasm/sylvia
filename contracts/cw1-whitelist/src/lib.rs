mod contract;
mod error;
mod multitest;

#[cfg(not(feature = "library"))]
pub mod entry_points {
    use cosmwasm_std::{
        entry_point, from_slice, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    };

    use crate::{
        contract::{
            Cw1WhitelistContract, ExecMsg, ImplExecMsg, ImplQueryMsg, InstantiateMsg, QueryMsg,
        },
        error::ContractError,
    };

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
        match from_slice::<ExecMsg>(&msg) {
            Ok(msg) => msg.dispatch(&CONTRACT, (deps, env, info)),
            Err(_) => from_slice::<ImplExecMsg>(&msg)?.dispatch(&CONTRACT, (deps, env, info)),
        }
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: Binary) -> Result<Binary, ContractError> {
        match from_slice::<QueryMsg>(&msg) {
            Ok(msg) => msg.dispatch(&CONTRACT, (deps, env)),
            Err(_) => from_slice::<ImplQueryMsg>(&msg)?.dispatch(&CONTRACT, (deps, env)),
        }
    }
}

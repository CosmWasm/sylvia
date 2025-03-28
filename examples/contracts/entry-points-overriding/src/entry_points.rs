#[cfg(not(feature = "library"))]
use sylvia::cw_std::entry_point;
use sylvia::cw_std::{DepsMut, Env, MessageInfo, Response, StdResult};

use crate::contract::CounterContract;
use crate::messages::{CustomExecMsg, SudoMsg};

#[cfg_attr(not(feature = "library"), entry_point(crate = "sylvia::cw_std"))]
pub fn sudo(deps: DepsMut, _env: Env, _msg: SudoMsg) -> StdResult<Response> {
    CounterContract::new().counter.save(deps.storage, &3)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point(crate = "sylvia::cw_std"))]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CustomExecMsg,
) -> StdResult<Response> {
    msg.dispatch((deps, env, info))
}

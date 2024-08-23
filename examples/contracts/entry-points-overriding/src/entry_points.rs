use sylvia::cw_std::{entry_point, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::contract::CounterContract;
use crate::messages::{CustomExecMsg, SudoMsg};

#[entry_point]
pub fn sudo(deps: DepsMut, _env: Env, _msg: SudoMsg) -> StdResult<Response> {
    CounterContract::new().counter.save(deps.storage, &3)?;
    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CustomExecMsg,
) -> StdResult<Response> {
    msg.dispatch((deps, env, info))
}

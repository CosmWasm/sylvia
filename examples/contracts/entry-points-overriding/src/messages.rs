use cosmwasm_schema::cw_serde;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, StdResult};
use sylvia::types::ExecCtx;

use crate::contract::{ContractExecMsg, CounterContract};

#[cw_serde]
pub struct CountResponse {
    pub count: u32,
}

#[cw_serde]
pub enum SudoMsg {
    SetCountToThree {},
}

#[cw_serde]
pub enum UserExecMsg {
    IncreaseByOne {},
}

pub fn increase_by_one(ctx: ExecCtx) -> StdResult<Response> {
    CounterContract::new()
        .counter
        .update(ctx.deps.storage, |count| -> Result<u32, StdError> {
            Ok(count + 1)
        })?;
    Ok(Response::new())
}

#[cw_serde]
pub enum CustomExecMsg {
    ContractExec(ContractExecMsg),
    CustomExec(UserExecMsg),
}

impl CustomExecMsg {
    pub fn dispatch(self, ctx: (DepsMut, Env, MessageInfo)) -> StdResult<Response> {
        match self {
            CustomExecMsg::ContractExec(msg) => msg.dispatch(&CounterContract::new(), ctx),
            CustomExecMsg::CustomExec(_) => increase_by_one(ctx.into()),
        }
    }
}

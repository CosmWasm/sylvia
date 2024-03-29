use cosmwasm_std::{Response, StdResult};
use sylvia::types::InstantiateCtx;
use sylvia::{contract, schemars};

use super::receiver;
pub struct ReceiverContract {}

#[contract]
#[sv::messages(receiver as Receiver)]
impl ReceiverContract {
    pub const fn new() -> Self {
        Self {}
    }
    #[sv::msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::new())
    }
}

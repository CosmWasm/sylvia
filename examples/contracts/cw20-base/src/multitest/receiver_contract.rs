use sylvia::contract;
use sylvia::ctx::InstantiateCtx;
use sylvia::cw_std::{Response, StdResult};

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

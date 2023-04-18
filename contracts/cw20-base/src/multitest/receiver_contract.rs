use cosmwasm_std::{Response, StdResult};
use sylvia::{contract, schemars, types::InstantiateCtx};

use super::receiver;
pub struct ReceiverContract {}

#[contract]
#[messages(receiver as Receiver)]
impl ReceiverContract {
    pub const fn new() -> Self {
        Self {}
    }
    #[msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::new())
    }
}

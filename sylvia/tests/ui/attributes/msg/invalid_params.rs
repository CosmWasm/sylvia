#![allow(unused_imports)]
use sylvia::contract;
use sylvia::ctx::{InstantiateCtx, ReplyCtx};
use sylvia::cw_std::{Reply, Response, StdResult};

pub struct Contract;

#[contract]
impl Contract {
    pub fn new() -> Self {
        Self
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[sv::msg(reply, unknown_parameter)]
    fn reply(&self, _ctx: ReplyCtx, _reply: Reply) -> StdResult<Response> {
        Ok(Response::new())
    }
}

fn main() {}

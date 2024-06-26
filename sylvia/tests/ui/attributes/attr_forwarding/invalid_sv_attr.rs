#![allow(unused_imports)]
use sylvia::cw_std::{Response, StdResult};
use sylvia::types::{ExecCtx, InstantiateCtx};

pub struct Contract;

#[sylvia::contract]
#[sv::msg_attr(exec,)]
#[sv::msg_attr(random_msg, PartialOrd)]
#[sv::msg_attr(exec PartialOrd)]
#[sv::msg_attr(PartialOrd)]
impl Contract {
    pub const fn new() -> Self {
        Self
    }

    #[sv::msg(instantiate)]
    fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[sv::msg(exec)]
    fn exec(&self, _ctx: ExecCtx) -> StdResult<Response> {
        Ok(Response::new())
    }
}

fn main() {}

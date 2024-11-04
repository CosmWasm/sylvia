#![allow(unused_imports)]
use sylvia::contract;
use sylvia::ctx::InstantiateCtx;
use sylvia::cw_std::{Response, StdResult};

pub struct Contract;

#[contract]
#[sv::features(unknown_parameter)]
impl Contract {
    pub fn new() -> Self {
        Self
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::new())
    }
}

fn main() {}

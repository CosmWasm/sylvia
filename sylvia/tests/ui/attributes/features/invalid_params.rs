#![allow(unused_imports, deprecated)]
use sylvia::contract;
use sylvia::cw_std::{Response, StdResult};
use sylvia::types::InstantiateCtx;

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

#![allow(unused_imports)]
use sylvia::cw_std::{Response, StdResult};
use sylvia::types::InstantiateCtx;

pub struct Contract {}

#[sylvia::contract]
impl Contract {
    #[sv::msg(instantiate)]
    pub fn instantiate(&self, ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::new())
    }
}

fn main() {}

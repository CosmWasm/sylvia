#![allow(unused_imports)]
use sylvia::cw_std::{Response, StdResult};
use sylvia::types::InstantiateCtx;

pub struct Contract;

#[sylvia::contract]
impl Contract {
    pub const fn new(admin: String) -> Self {
        Contract
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(&self, ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::new())
    }
}

fn main() {}

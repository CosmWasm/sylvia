#![allow(unused_imports)]
use sylvia::cw_std::{Response, StdResult};
use sylvia::types::InstantiateCtx;

#[sylvia::cw_schema::cw_serde]
pub struct MyMsg;
impl sylvia::cw_std::CustomMsg for MyMsg {}

pub struct Contract;

#[sylvia::contract]
#[sv::custom(mgs=MyMsg)]
impl Contract {
    pub const fn new() -> Self {
        Self
    }

    #[sv::msg(instantiate)]
    fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response<MyMsg>> {
        Ok(Response::new())
    }
}

fn main() {}

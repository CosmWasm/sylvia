#![allow(unused_imports)]
use sylvia::cw_std::{Response, StdResult};
use sylvia::types::InstantiateCtx;

mod interface {
    use sylvia::cw_std::StdError;

    #[sylvia::interface]
    pub trait Interface {
        type Error;
    }

    impl Interface for crate::Contract {
        type Error = StdError;
    }
}

pub struct Contract;

#[sylvia::contract]
#[sv::messages(interface: custom(wrong))]
impl Contract {
    pub const fn new() -> Self {
        Contract
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(&self, ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::new())
    }
}

fn main() {}

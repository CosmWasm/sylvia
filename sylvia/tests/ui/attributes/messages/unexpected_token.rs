#![allow(unused_imports)]
use sylvia::cw_std::{Empty, Response, StdResult};
use sylvia::types::InstantiateCtx;

mod interface {
    use sylvia::cw_std::{Empty, StdError};
    use sylvia::types::CustomMsg;

    #[sylvia::interface]
    pub trait Interface {
        type Error;
        type ParamT: CustomMsg;
    }

    impl Interface for crate::Contract {
        type Error = StdError;
        type ParamT = Empty;
    }
}

pub struct Contract;

#[sylvia::contract]
#[sv::messages(interface(Empty))]
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

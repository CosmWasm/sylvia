#![allow(unused_imports)]
use sylvia::cw_std::{Response, StdResult};
use sylvia::types::InstantiateCtx;

pub struct Contract;


mod interface {
    use sylvia::cw_std::{Response, StdResult, StdError};
    use sylvia::types::{InstantiateCtx, MigrateCtx};

    #[sylvia::interface]
    trait Interface {
        type Error: From<StdError>;

        #[sv::msg(instantiate)]
        fn instantiate(&self, ctx: InstantiateCtx) -> StdResult<Response>;

        #[sv::msg(migrate)]
        fn migrate(&self, ctx: MigrateCtx) -> StdResult<Response>;
    }
}

#[sylvia::contract]
impl Contract {
    pub const fn new() -> Self {
        Contract
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(&self, ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[sv::msg(instantiate)]
    pub fn instantiate2(&self, ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::new())
    }
}

fn main() {}

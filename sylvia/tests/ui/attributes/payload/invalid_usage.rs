#![allow(unused_imports, unused_variables)]
use sylvia::ctx::{InstantiateCtx, ReplyCtx};
use sylvia::cw_std::{Addr, Binary, Response, StdResult};

pub mod used_on_context {
    use super::*;

    pub struct Contract {}

    #[sylvia::contract]
    impl Contract {
        pub const fn new() -> Self {
            Self {}
        }

        #[sv::msg(instantiate)]
        fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(reply, reply_on=success)]
        fn reply(&self, #[sv::payload(raw)] _ctx: ReplyCtx) -> StdResult<Response> {
            Ok(Response::new())
        }
    }
}

pub mod used_on_self {
    use super::*;

    pub struct Contract {}

    #[sylvia::contract]
    impl Contract {
        pub const fn new() -> Self {
            Self {}
        }

        #[sv::msg(instantiate)]
        fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(reply, reply_on=success)]
        fn reply(
            #[sv::payload(raw)] &self,
            _ctx: ReplyCtx,
            payload: Binary,
        ) -> StdResult<Response> {
            Ok(Response::new())
        }
    }
}

fn main() {}

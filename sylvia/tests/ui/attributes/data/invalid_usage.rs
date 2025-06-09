#![allow(unused_imports, unused_variables)]
use sylvia::ctx::{InstantiateCtx, ReplyCtx};
use sylvia::cw_std::{Addr, Binary, Response, StdResult};

pub mod attributes_swapped {
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
            &self,
            _ctx: ReplyCtx,
            #[sv::payload(raw)] param: Binary,
            #[sv::data(opt, raw)] _data: Option<Binary>,
        ) -> StdResult<Response> {
            Ok(Response::new())
        }
    }
}

pub mod error_handler {
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

        #[sv::msg(reply, reply_on=error)]
        fn reply(
            &self,
            _ctx: ReplyCtx,
            #[sv::data(opt, raw)] _data: Option<Binary>,
            #[sv::payload(raw)] param: Binary,
        ) -> StdResult<Response> {
            Ok(Response::new())
        }
    }
}

fn main() {}

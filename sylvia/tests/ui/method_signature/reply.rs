#![allow(unused_imports)]
use sylvia::cw_std::{Binary, Response, StdResult};
use sylvia::types::{InstantiateCtx, ReplyCtx};

pub mod mismatched_params {
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

        #[sv::msg(reply, handlers=[on_instantiated], reply_on=success)]
        fn first_reply(
            &self,
            _ctx: ReplyCtx,
            _data: Option<Binary>,
            param: String,
        ) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(reply, handlers=[on_instantiated], reply_on=failure)]
        fn second_reply(
            &self,
            _ctx: ReplyCtx,
            _data: Option<Binary>,
            param: u32,
        ) -> StdResult<Response> {
            Ok(Response::new())
        }
    }
}

pub mod mismatched_param_arity {
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

        #[sv::msg(reply, handlers=[on_instantiated], reply_on=success)]
        fn first_reply(
            &self,
            _ctx: ReplyCtx,
            _data: Option<Binary>,
            param: String,
        ) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(reply, handlers=[on_instantiated], reply_on=failure)]
        fn second_reply(
            &self,
            _ctx: ReplyCtx,
            _data: Option<Binary>,
            param: String,
            param: u32,
        ) -> StdResult<Response> {
            Ok(Response::new())
        }
    }
}

fn main() {}

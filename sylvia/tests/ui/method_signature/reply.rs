#![allow(unused_imports, unused_variables)]
use sylvia::ctx::{InstantiateCtx, ReplyCtx};
use sylvia::cw_std::{Addr, Binary, Response, StdResult};

pub mod mismatched_params {
    use super::*;

    pub struct Contract {}

    #[sylvia::contract]
    #[sv::features(replies)]
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
            #[sv::data(opt, raw)] _data: Option<Binary>,
            param: String,
        ) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(reply, handlers=[on_instantiated], reply_on=error)]
        fn second_reply(&self, _ctx: ReplyCtx, error: String, param: u32) -> StdResult<Response> {
            Ok(Response::new())
        }
    }
}

pub mod mismatched_param_arity {
    use super::*;

    pub struct Contract {}

    #[sylvia::contract]
    #[sv::features(replies)]
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
            #[sv::data(opt, raw)] _data: Option<Binary>,
            param: String,
        ) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(reply, handlers=[on_instantiated], reply_on=error)]
        fn second_reply(
            &self,
            _ctx: ReplyCtx,
            error: String,
            param: String,
            param: u32,
        ) -> StdResult<Response> {
            Ok(Response::new())
        }
    }
}

pub mod redundant_params {
    use super::*;

    pub struct Contract {}

    #[sylvia::contract]
    #[sv::features(replies)]
    impl Contract {
        pub const fn new() -> Self {
            Self {}
        }

        #[sv::msg(instantiate)]
        fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(reply, reply_on=success)]
        fn first_reply(
            &self,
            _ctx: ReplyCtx,
            redundant_before1: u32,
            redundant_before2: String,
            #[sv::data(opt, raw)] _data: Option<Binary>,
            #[sv::payload(raw)] param: String,
        ) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(reply, reply_on=success)]
        fn second_reply(
            &self,
            _ctx: ReplyCtx,
            #[sv::data(opt, raw)] _data: Option<Binary>,
            redundant_between1: u32,
            redudnant_between2: String,
            #[sv::payload(raw)] param: String,
            redundant_after1: Binary,
            redundant_after2: Addr,
        ) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(reply, reply_on=success)]
        fn third_reply(
            &self,
            _ctx: ReplyCtx,
            #[sv::data(opt, raw)] _data: Option<Binary>,
            #[sv::payload(raw)] param: String,
            redundant: Binary,
        ) -> StdResult<Response> {
            Ok(Response::new())
        }
    }
}

fn main() {}

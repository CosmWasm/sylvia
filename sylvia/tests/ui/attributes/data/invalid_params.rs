#![allow(unused_imports)]

use sylvia::contract;
use sylvia::ctx::{InstantiateCtx, ReplyCtx};
use sylvia::cw_std::{Binary, Reply, Response, StdResult};

pub mod invalid_data {
    use super::*;

    pub struct Contract;

    #[contract]
    impl Contract {
        pub fn new() -> Self {
            Self
        }

        #[sv::msg(instantiate)]
        pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(reply, reply_on=success)]
        fn reply(
            &self,
            _ctx: ReplyCtx,
            #[sv::data(invalid)] _data: Option<Binary>,
            _param: String,
        ) -> StdResult<Response> {
            Ok(Response::new())
        }
    }
}

pub mod instantiate_and_raw {
    use super::*;

    pub struct Contract;

    #[contract]
    impl Contract {
        pub fn new() -> Self {
            Self
        }

        #[sv::msg(instantiate)]
        pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(reply, reply_on=success)]
        fn reply(
            &self,
            _ctx: ReplyCtx,
            #[sv::data(instantiate, raw)] _data: Option<Binary>,
            _param: String,
        ) -> StdResult<Response> {
            Ok(Response::new())
        }
    }
}

fn main() {}

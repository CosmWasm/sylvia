#![allow(unused_imports)]
#![cfg(feature = "sv_replies")]

use sylvia::contract;
use sylvia::cw_std::{Reply, Response, StdResult};
use sylvia::types::{InstantiateCtx, ReplyCtx};

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

    #[sv::msg(reply, handlers(handler1))]
    fn reply_always(&self, _ctx: ReplyCtx, _reply: Reply) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[sv::msg(reply, handlers(handler1), reply_on=success)]
    fn duplicated_success_for_reply_always(
        &self,
        _ctx: ReplyCtx,
        _reply: Reply,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[sv::msg(reply, handlers(handler2), reply_on=failure)]
    fn some_reply(&self, _ctx: ReplyCtx, _reply: Reply) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[sv::msg(reply, reply_on=failure)]
    fn handler2(&self, _ctx: ReplyCtx, _reply: Reply) -> StdResult<Response> {
        Ok(Response::new())
    }
}

fn main() {}

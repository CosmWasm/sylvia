#![allow(unused_imports)]

use sylvia::contract;
use sylvia::ctx::{InstantiateCtx, ReplyCtx};
use sylvia::cw_std::{Reply, Response, StdResult};

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

    #[sv::msg(reply, handlers=[handler1])]
    fn reply_always(
        &self,
        _ctx: ReplyCtx,
        _result: SubMsgResult,
        #[sv::payload(raw)] payload: Binary,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[sv::msg(reply, handlers=[handler1], reply_on=success)]
    fn duplicated_success_for_reply_always(
        &self,
        _ctx: ReplyCtx,
        #[sv::payload(raw)] reply: Binary,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[sv::msg(reply, handlers=[handler2], reply_on=error)]
    fn some_reply(
        &self,
        _ctx: ReplyCtx,
        error: String,
        #[sv::payload(raw)] payload: Binary,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[sv::msg(reply, reply_on=error)]
    fn handler2(
        &self,
        _ctx: ReplyCtx,
        error: String,
        #[sv::payload(raw)] payload: Binary,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }
}

fn main() {}

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

    #[sv::msg(reply)]
    fn clean(&self, _ctx: ReplyCtx, _reply: Reply) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[allow(dead_code)]
    #[sv::msg(reply, handlers(handler_one, handler_two))]
    fn custom_handlers(&self, _ctx: ReplyCtx, _reply: Reply) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[allow(dead_code)]
    #[sv::msg(reply, reply_on = success)]
    fn reply_on(&self, _ctx: ReplyCtx, _reply: Reply) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[allow(dead_code)]
    #[sv::msg(reply, reply_on = always)]
    fn reply_on_always(&self, _ctx: ReplyCtx, _reply: Reply) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[allow(dead_code)]
    #[sv::msg(reply, handlers(handler_one, handler_two), reply_on = failure)]
    fn both_parameters(&self, _ctx: ReplyCtx, _reply: Reply) -> StdResult<Response> {
        Ok(Response::new())
    }
}

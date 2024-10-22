#![allow(unused_imports, deprecated)]

use sylvia::contract;
use sylvia::cw_std::{Binary, Reply, Response, StdResult};
use sylvia::types::{InstantiateCtx, ReplyCtx};

pub struct Contract;

#[contract]
#[sv::features(replies)]
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
        #[sv::data(raw, opt)] _data: Option<Binary>,
        #[sv::payload(invalid)] _param: Option<Binary>,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }
}

fn main() {}

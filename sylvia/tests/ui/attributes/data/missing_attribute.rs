#![allow(unused_imports)]

use sylvia::contract;
use sylvia::cw_std::{Reply, Response, StdResult};
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
        // If the `data` attribute is missing, the data field should be omitted.
        _data: Option<Binary>,
        param: String,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }
}

fn main() {}

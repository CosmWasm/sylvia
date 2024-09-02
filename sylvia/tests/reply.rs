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
    #[sv::msg(reply, handlers=[handler_one, handler_two])]
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
    #[sv::msg(reply, handlers=[handler_one, handler_two], reply_on = failure)]
    fn both_parameters(&self, _ctx: ReplyCtx, _reply: Reply) -> StdResult<Response> {
        Ok(Response::new())
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    #[test]
    fn reply_id_generation() {
        // Assert IDs uniqueness
        let unique_ids_count = [
            super::sv::CLEAN_REPLY_ID,
            super::sv::HANDLER_ONE_REPLY_ID,
            super::sv::HANDLER_TWO_REPLY_ID,
            super::sv::REPLY_ON_REPLY_ID,
            super::sv::REPLY_ON_ALWAYS_REPLY_ID,
        ]
        .iter()
        .unique()
        .count();

        assert_eq!(unique_ids_count, 5);

        assert_eq!(super::sv::CLEAN_REPLY_ID, 0);
        assert_eq!(super::sv::HANDLER_ONE_REPLY_ID, 1);
        assert_eq!(super::sv::HANDLER_TWO_REPLY_ID, 2);
        assert_eq!(super::sv::REPLY_ON_REPLY_ID, 3);
        assert_eq!(super::sv::REPLY_ON_ALWAYS_REPLY_ID, 4);
    }
}

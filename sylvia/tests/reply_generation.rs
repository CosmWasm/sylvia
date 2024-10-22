use cosmwasm_std::{Binary, SubMsgResult};
use itertools::Itertools;
use sylvia::cw_std::{Response, StdResult};
#[allow(deprecated)]
use sylvia::replies::ReplyCtx;
use sylvia::types::InstantiateCtx;
use sylvia::{contract, entry_points};

pub struct Contract;

#[entry_points]
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

    #[sv::msg(reply)]
    #[allow(deprecated)]
    fn clean(
        &self,
        _ctx: ReplyCtx,
        _result: SubMsgResult,
        #[sv::payload(raw)] _payload: Binary,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[allow(dead_code, deprecated)]
    #[sv::msg(reply, handlers=[handler_one, handler_two])]
    fn custom_handlers(
        &self,
        _ctx: ReplyCtx,
        _result: SubMsgResult,
        #[sv::payload(raw)] _payload: Binary,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[allow(dead_code, deprecated)]
    #[sv::msg(reply, reply_on = success)]
    fn two_handlers(
        &self,
        _ctx: ReplyCtx,
        #[sv::data(raw, opt)] _data: Option<Binary>,
        #[sv::payload(raw)] _payload: Binary,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[allow(dead_code, deprecated)]
    #[sv::msg(reply, reply_on = always)]
    fn reply_on_always(
        &self,
        _ctx: ReplyCtx,
        _result: SubMsgResult,
        #[sv::payload(raw)] _payload: Binary,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[allow(dead_code, deprecated)]
    #[sv::msg(reply, handlers=[two_handlers], reply_on = error)]
    fn both_parameters(
        &self,
        _ctx: ReplyCtx,
        _error: String,
        #[sv::payload(raw)] _payload: Binary,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }
}

#[test]
fn reply_id_generation() {
    // Assert IDs uniqueness
    let unique_ids_count = [
        sv::CLEAN_REPLY_ID,
        sv::HANDLER_ONE_REPLY_ID,
        sv::HANDLER_TWO_REPLY_ID,
        sv::TWO_HANDLERS_REPLY_ID,
        sv::REPLY_ON_ALWAYS_REPLY_ID,
    ]
    .iter()
    .unique()
    .count();

    assert_eq!(unique_ids_count, 5);

    assert_eq!(sv::CLEAN_REPLY_ID, 0);
    assert_eq!(sv::HANDLER_ONE_REPLY_ID, 1);
    assert_eq!(sv::HANDLER_TWO_REPLY_ID, 2);
    assert_eq!(sv::TWO_HANDLERS_REPLY_ID, 3);
    assert_eq!(sv::REPLY_ON_ALWAYS_REPLY_ID, 4);
}

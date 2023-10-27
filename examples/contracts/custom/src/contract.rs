use cosmwasm_std::{CosmosMsg, QueryRequest, Response, StdResult};
use sylvia::types::{ExecCtx, InstantiateCtx, QueryCtx};
use sylvia::{contract, schemars};

#[cfg(not(feature = "library"))]
use sylvia::entry_points;

use crate::messages::{CountResponse, CounterMsg, CounterQuery};

pub struct CustomContract;

#[cfg_attr(not(feature = "library"), entry_points)]
#[contract]
#[messages(cw1 as Cw1: custom(msg, query))]
#[sv::custom(query=CounterQuery, msg=CounterMsg)]
impl CustomContract {
    pub const fn new() -> Self {
        Self
    }

    #[msg(instantiate)]
    pub fn instantiate(
        &self,
        _ctx: InstantiateCtx<CounterQuery>,
    ) -> StdResult<Response<CounterMsg>> {
        Ok(Response::default())
    }

    #[msg(exec)]
    pub fn send_custom(&self, _ctx: ExecCtx<CounterQuery>) -> StdResult<Response<CounterMsg>> {
        let msg = CosmosMsg::Custom(CounterMsg::Increment {});
        let resp = Response::default().add_message(msg);
        Ok(resp)
    }

    #[msg(query)]
    pub fn query_custom(&self, ctx: QueryCtx<CounterQuery>) -> StdResult<CountResponse> {
        let resp = ctx
            .deps
            .querier
            .query::<CountResponse>(&QueryRequest::Custom(CounterQuery::Count {}))?;

        Ok(resp)
    }
}

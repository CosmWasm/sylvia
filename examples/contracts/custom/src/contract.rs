use cosmwasm_std::{CosmosMsg, QueryRequest, Response, StdResult};
use cw_storage_plus::Item;
use sylvia::types::{ExecCtx, InstantiateCtx, QueryCtx, SudoCtx};
use sylvia::{contract, schemars};

#[cfg(not(feature = "library"))]
use sylvia::entry_points;

use crate::messages::{CountResponse, CounterMsg, CounterQuery};

pub struct CustomContract {
    pub(crate) sudo_counter: Item<u64>,
}

#[cfg_attr(not(feature = "library"), entry_points)]
#[contract]
#[sv::messages(cw1 as Cw1: custom(msg, query))]
#[sv::custom(query=CounterQuery, msg=CounterMsg)]
impl CustomContract {
    pub const fn new() -> Self {
        Self {
            sudo_counter: Item::new("sudo_counter"),
        }
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: InstantiateCtx<CounterQuery>,
    ) -> StdResult<Response<CounterMsg>> {
        self.sudo_counter.save(ctx.deps.storage, &0)?;
        Ok(Response::default())
    }

    #[sv::msg(exec)]
    pub fn send_custom(&self, _ctx: ExecCtx<CounterQuery>) -> StdResult<Response<CounterMsg>> {
        let msg = CosmosMsg::Custom(CounterMsg::Increment {});
        let resp = Response::default().add_message(msg);
        Ok(resp)
    }

    #[sv::msg(query)]
    pub fn query_custom(&self, ctx: QueryCtx<CounterQuery>) -> StdResult<CountResponse> {
        let resp = ctx
            .deps
            .querier
            .query::<CountResponse>(&QueryRequest::Custom(CounterQuery::Count {}))?;

        Ok(resp)
    }

    #[sv::msg(query)]
    pub fn sudo_counter(&self, ctx: QueryCtx<CounterQuery>) -> StdResult<CountResponse> {
        let count = self.sudo_counter.load(ctx.deps.storage)?;

        Ok(CountResponse { count })
    }

    #[sv::msg(sudo)]
    pub fn increment_sudo_counter(
        &self,
        ctx: SudoCtx<CounterQuery>,
    ) -> StdResult<Response<CounterMsg>> {
        self.sudo_counter
            .update(ctx.deps.storage, |value| -> StdResult<_> { Ok(value + 1) })?;
        Ok(Response::new())
    }
}

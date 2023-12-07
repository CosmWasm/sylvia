use cosmwasm_std::{CosmosMsg, Response, StdError, StdResult};
use generic::Generic;
use sylvia::contract;
use sylvia::types::{ExecCtx, QueryCtx, SvCustomMsg};

type QueryAlias1 = SvCustomMsg;
type QueryAlias2 = SvCustomMsg;
type QueryAlias3 = SvCustomMsg;

#[contract(module = crate::contract)]
#[messages(generic as Generic)]
#[sv::custom(msg = SvCustomMsg)]
impl
    Generic<
        SvCustomMsg,
        SvCustomMsg,
        SvCustomMsg,
        QueryAlias1,
        QueryAlias2,
        QueryAlias3,
        sylvia::types::SvCustomMsg,
    > for crate::contract::NonGenericContract
{
    type Error = StdError;

    #[msg(exec)]
    fn generic_exec_one(
        &self,
        _ctx: ExecCtx,
        _msgs1: Vec<CosmosMsg<sylvia::types::SvCustomMsg>>,
        _msgs2: Vec<CosmosMsg<sylvia::types::SvCustomMsg>>,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(exec)]
    fn generic_exec_two(
        &self,
        _ctx: ExecCtx,
        _msgs1: Vec<CosmosMsg<sylvia::types::SvCustomMsg>>,
        _msgs2: Vec<CosmosMsg<sylvia::types::SvCustomMsg>>,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    // Sylvia will fail if single type is used to match against two different generics
    // It's because we have to map unique generics used as they can be used multiple times.
    // If for some reason like here one type would be used in place of two generics either full
    // path or some alias has to be used.
    #[msg(query)]
    fn generic_query_one(
        &self,
        _ctx: QueryCtx,
        _msg1: QueryAlias1,
        _msg2: QueryAlias2,
    ) -> StdResult<SvCustomMsg> {
        Ok(SvCustomMsg {})
    }

    #[msg(query)]
    fn generic_query_two(
        &self,
        _ctx: QueryCtx,
        _msg1: QueryAlias2,
        _msg2: QueryAlias3,
    ) -> StdResult<SvCustomMsg> {
        Ok(SvCustomMsg {})
    }
}

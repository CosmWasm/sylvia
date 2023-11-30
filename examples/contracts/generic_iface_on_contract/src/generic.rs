use cosmwasm_std::{CosmosMsg, Response, StdError, StdResult};
use generic::Generic;
use sylvia::contract;
use sylvia::types::{ExecCtx, QueryCtx, SvCustomMsg};

#[contract(module = crate::contract)]
#[messages(generic as Generic)]
#[sv::custom(msg = SvCustomMsg)]
impl Generic<SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, sylvia::types::SvCustomMsg>
    for crate::contract::NonGenericContract
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
    fn generic_query(
        &self,
        _ctx: QueryCtx,
        _msg: sylvia::types::SvCustomMsg,
    ) -> StdResult<SvCustomMsg> {
        Ok(SvCustomMsg {})
    }
}

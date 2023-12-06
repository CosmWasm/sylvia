use cosmwasm_std::{CosmosMsg, Response, StdError, StdResult};
use custom_and_generic::CustomAndGeneric;
use sylvia::contract;
use sylvia::types::{ExecCtx, QueryCtx, SvCustomMsg, SvCustomQuery};

type QueryAlias1 = SvCustomMsg;
type QueryAlias2 = SvCustomMsg;
type QueryAlias3 = SvCustomMsg;

#[contract(module = crate::contract)]
#[messages(custom_and_generic as CustomAndGeneric)]
#[sv::custom(msg=sylvia::types::SvCustomMsg, query=SvCustomQuery)]
impl
    CustomAndGeneric<
        SvCustomMsg,
        SvCustomMsg,
        SvCustomMsg,
        QueryAlias1,
        QueryAlias2,
        QueryAlias3,
        sylvia::types::SvCustomMsg,
        sylvia::types::SvCustomMsg,
        SvCustomQuery,
    > for crate::contract::NonGenericContract
{
    type Error = StdError;

    #[msg(exec)]
    fn custom_generic_execute_one(
        &self,
        _ctx: ExecCtx<SvCustomQuery>,
        _msgs1: Vec<CosmosMsg<sylvia::types::SvCustomMsg>>,
        _msgs2: Vec<CosmosMsg<sylvia::types::SvCustomMsg>>,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[msg(exec)]
    fn custom_generic_execute_two(
        &self,
        _ctx: ExecCtx<SvCustomQuery>,
        _msgs1: Vec<CosmosMsg<sylvia::types::SvCustomMsg>>,
        _msgs2: Vec<CosmosMsg<sylvia::types::SvCustomMsg>>,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[msg(query)]
    fn custom_generic_query_one(
        &self,
        _ctx: QueryCtx<SvCustomQuery>,
        _msg1: QueryAlias1,
        _msg2: QueryAlias2,
    ) -> StdResult<SvCustomMsg> {
        Ok(SvCustomMsg {})
    }

    #[msg(query)]
    fn custom_generic_query_two(
        &self,
        _ctx: QueryCtx<SvCustomQuery>,
        _msg1: QueryAlias2,
        _msg2: QueryAlias3,
    ) -> StdResult<SvCustomMsg> {
        Ok(SvCustomMsg {})
    }
}

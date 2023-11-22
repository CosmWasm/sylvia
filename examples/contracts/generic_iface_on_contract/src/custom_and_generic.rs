use cosmwasm_std::{CosmosMsg, Response, StdError, StdResult};
use custom_and_generic::CustomAndGeneric;
use sylvia::contract;
use sylvia::types::{ExecCtx, QueryCtx, SvCustomMsg, SvCustomQuery};

#[contract(module = crate::contract)]
#[messages(custom_and_generic as CustomAndGeneric)]
#[sv::custom(msg=sylvia::types::SvCustomMsg, query=SvCustomQuery)]
impl
    CustomAndGeneric<
        SvCustomMsg,
        SvCustomMsg,
        SvCustomMsg,
        sylvia::types::SvCustomMsg,
        SvCustomQuery,
    > for crate::contract::NonGenericContract
{
    type Error = StdError;

    #[msg(exec)]
    fn custom_generic_execute(
        &self,
        _ctx: ExecCtx<SvCustomQuery>,
        _msgs: Vec<CosmosMsg<sylvia::types::SvCustomMsg>>,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[msg(query)]
    fn custom_generic_query(
        &self,
        _ctx: QueryCtx<SvCustomQuery>,
        _msg: sylvia::types::SvCustomMsg,
    ) -> StdResult<SvCustomMsg> {
        Ok(SvCustomMsg {})
    }
}

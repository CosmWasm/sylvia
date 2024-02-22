use cosmwasm_std::{CosmosMsg, Response, StdError, StdResult};
use custom_and_generic::CustomAndGeneric;
use sylvia::contract;
use sylvia::types::{ExecCtx, QueryCtx, SudoCtx, SvCustomMsg, SvCustomQuery};

#[contract(module = crate::contract)]
#[sv::messages(custom_and_generic as CustomAndGeneric)]
#[sv::custom(msg=sylvia::types::SvCustomMsg, query=SvCustomQuery)]
impl CustomAndGeneric for crate::contract::NonGenericContract {
    type Error = StdError;
    type Exec1T = SvCustomMsg;
    type Exec2T = SvCustomMsg;
    type Exec3T = SvCustomMsg;
    type Query1T = SvCustomMsg;
    type Query2T = SvCustomMsg;
    type Query3T = SvCustomMsg;
    type Sudo1T = SvCustomMsg;
    type Sudo2T = SvCustomMsg;
    type Sudo3T = SvCustomMsg;
    type ExecC = SvCustomMsg;
    type QueryC = SvCustomQuery;
    type RetT = SvCustomMsg;

    #[sv::msg(exec)]
    fn custom_generic_execute_one(
        &self,
        _ctx: ExecCtx<Self::QueryC>,
        _msgs1: Vec<CosmosMsg<Self::Exec1T>>,
        _msgs2: Vec<CosmosMsg<Self::Exec2T>>,
    ) -> StdResult<Response<Self::ExecC>> {
        Ok(Response::new())
    }

    #[sv::msg(exec)]
    fn custom_generic_execute_two(
        &self,
        _ctx: ExecCtx<Self::QueryC>,
        _msgs1: Vec<CosmosMsg<Self::Exec2T>>,
        _msgs2: Vec<CosmosMsg<Self::Exec3T>>,
    ) -> StdResult<Response<Self::ExecC>> {
        Ok(Response::new())
    }

    #[sv::msg(query)]
    fn custom_generic_query_one(
        &self,
        _ctx: QueryCtx<Self::QueryC>,
        _msg1: Self::Query1T,
        _msg2: Self::Query2T,
    ) -> StdResult<Self::RetT> {
        Ok(SvCustomMsg {})
    }

    #[sv::msg(query)]
    fn custom_generic_query_two(
        &self,
        _ctx: QueryCtx<Self::QueryC>,
        _msg1: Self::Query2T,
        _msg2: Self::Query3T,
    ) -> StdResult<Self::RetT> {
        Ok(SvCustomMsg {})
    }

    #[sv::msg(sudo)]
    fn custom_generic_sudo_one(
        &self,
        _ctx: SudoCtx<Self::QueryC>,
        _msgs1: CosmosMsg<Self::Sudo1T>,
        _msgs2: CosmosMsg<Self::Sudo2T>,
    ) -> StdResult<Response<Self::ExecC>> {
        Ok(Response::new())
    }

    #[sv::msg(sudo)]
    fn custom_generic_sudo_two(
        &self,
        _ctx: SudoCtx<Self::QueryC>,
        _msgs1: CosmosMsg<Self::Sudo2T>,
        _msgs2: CosmosMsg<Self::Sudo3T>,
    ) -> StdResult<Response<Self::ExecC>> {
        Ok(Response::new())
    }
}

use cosmwasm_std::{Response, StdResult};
use std::marker::PhantomData;

use sylvia::types::{
    CustomMsg, CustomQuery, ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx, SudoCtx,
};
use sylvia_derive::contract;

pub struct SomeContract<Instantiate, Query, Exec, Migrate, Sudo, Ret, CMsg, CQuery> {
    #[allow(clippy::complexity)]
    _phantom: PhantomData<(Instantiate, Query, Exec, Migrate, Sudo, Ret, CMsg, CQuery)>,
}

#[allow(dead_code)]
#[contract]
#[sv::custom(msg=CMsg, query=CQuery)]
impl<Instantiate, Query, Exec, Migrate, Sudo, Ret, CMsg, CQuery>
    SomeContract<Instantiate, Query, Exec, Migrate, Sudo, Ret, CMsg, CQuery>
where
    Instantiate: CustomMsg + 'static,
    Query: CustomMsg + 'static,
    Exec: CustomMsg + 'static,
    Migrate: CustomMsg + 'static,
    Sudo: CustomMsg + 'static,
    Ret: CustomMsg + 'static,
    CMsg: CustomMsg + 'static,
    CQuery: CustomQuery + 'static,
{
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(
        &self,
        _ctx: InstantiateCtx<CQuery>,
        _param: Instantiate,
    ) -> StdResult<Response<CMsg>> {
        Ok(Response::new())
    }

    #[sv::msg(exec)]
    pub fn exec(&self, _ctx: ExecCtx<CQuery>, _param: Exec) -> StdResult<Response<CMsg>> {
        Ok(Response::new())
    }

    #[sv::msg(query)]
    pub fn query(&self, _ctx: QueryCtx<CQuery>, _param: Query) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[sv::msg(sudo)]
    pub fn sudo(&self, _ctx: SudoCtx<CQuery>, _param: Sudo) -> StdResult<Response<CMsg>> {
        Ok(Response::new())
    }

    #[sv::msg(migrate)]
    pub fn migrate(&self, _ctx: MigrateCtx<CQuery>, _param: Migrate) -> StdResult<Response<CMsg>> {
        Ok(Response::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::SomeContract;
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{Addr, QuerierWrapper};
    use sylvia::types::ContractApi;

    #[cw_serde]
    pub struct SvCustomMsg;
    impl cosmwasm_std::CustomMsg for SvCustomMsg {}
    #[cw_serde]
    pub struct SvCustomQuery;
    impl cosmwasm_std::CustomQuery for SvCustomQuery {}

    pub type ConcreteContract = SomeContract<
        SvCustomMsg,
        SvCustomMsg,
        SvCustomMsg,
        SvCustomMsg,
        SvCustomMsg,
        SvCustomMsg,
        SvCustomMsg,
        SvCustomQuery,
    >;

    #[test]
    fn api() {
        let owner = Addr::unchecked("owner");

        // Messages
        let _: crate::sv::InstantiateMsg<SvCustomMsg> =
            <ConcreteContract as ContractApi>::Instantiate::new(SvCustomMsg);

        let exec: crate::sv::ExecMsg<SvCustomMsg> =
            <ConcreteContract as ContractApi>::Exec::exec(SvCustomMsg);

        let query: crate::sv::QueryMsg<SvCustomMsg> =
            <ConcreteContract as ContractApi>::Query::query(SvCustomMsg);

        let sudo: crate::sv::SudoMsg<SvCustomMsg> =
            <ConcreteContract as ContractApi>::Sudo::sudo(SvCustomMsg);

        let _: crate::sv::ContractExecMsg<_, _, _, _, _, _, _, _> =
            <ConcreteContract as ContractApi>::ContractExec::SomeContract(exec);

        let _: crate::sv::ContractQueryMsg<_, _, _, _, _, _, _, _> =
            <ConcreteContract as ContractApi>::ContractQuery::SomeContract(query);

        let _: crate::sv::ContractSudoMsg<_, _, _, _, _, _, _, _> =
            <ConcreteContract as ContractApi>::ContractSudo::SomeContract(sudo);

        // Communication
        let _: sylvia::types::Remote<'_, _> =
            <ConcreteContract as ContractApi>::Remote::new(owner.clone());

        let deps = mock_dependencies();
        let querier_wrapper = QuerierWrapper::new(&deps.querier);
        let _: sylvia::types::BoundQuerier<'_, SvCustomQuery, _> =
            <ConcreteContract as ContractApi>::Querier::borrowed(&owner, &querier_wrapper);

        // Customs
        let _: <ConcreteContract as ContractApi>::CustomMsg = SvCustomMsg {};
        let _: <ConcreteContract as ContractApi>::CustomQuery = SvCustomQuery {};
    }
}

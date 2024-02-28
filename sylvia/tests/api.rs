use cosmwasm_std::{Response, StdResult};
use std::marker::PhantomData;

use sylvia::types::{CustomMsg, ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx};
use sylvia_derive::contract;

pub struct SomeContract<Instantiate, Query, Exec, Migrate, Ret> {
    _phantom: PhantomData<(Instantiate, Query, Exec, Migrate, Ret)>,
}

#[contract]
impl<Instantiate, Query, Exec, Migrate, Ret> SomeContract<Instantiate, Query, Exec, Migrate, Ret>
where
    Instantiate: CustomMsg + 'static,
    Query: CustomMsg + 'static,
    Exec: CustomMsg + 'static,
    Migrate: CustomMsg + 'static,
    Ret: CustomMsg + 'static,
{
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx, _param: Instantiate) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[sv::msg(exec)]
    pub fn exec(&self, _ctx: ExecCtx, _param: Exec) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[sv::msg(query)]
    pub fn query(&self, _ctx: QueryCtx, _param: Query) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[sv::msg(migrate)]
    pub fn migrate(&self, _ctx: MigrateCtx, _param: Migrate) -> StdResult<Response> {
        Ok(Response::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::SomeContract;
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{Addr, QuerierWrapper};
    use sylvia::types::{ContractApi, SvCustomMsg};

    #[test]
    fn api() {
        let owner = Addr::unchecked("owner");

        let _: crate::sv::InstantiateMsg<SvCustomMsg> = <SomeContract<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
        > as ContractApi>::Instantiate::new(
            SvCustomMsg
        );

        let exec: crate::sv::ExecMsg<SvCustomMsg> = <SomeContract<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
        > as ContractApi>::Exec::exec(
            SvCustomMsg
        );

        let query: crate::sv::QueryMsg<SvCustomMsg> = <SomeContract<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
        > as ContractApi>::Query::query(
            SvCustomMsg
        );

        let _: crate::sv::ContractExecMsg<SvCustomMsg> = <SomeContract<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
        > as ContractApi>::ContractExec::SomeContract(
            exec
        );

        let _: crate::sv::ContractQueryMsg<SvCustomMsg> = <SomeContract<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
        > as ContractApi>::ContractQuery::SomeContract(
            query
        );

        let _: sylvia::types::Remote<'_, _> = <SomeContract<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
        > as ContractApi>::Remote::new(owner.clone());

        let deps = mock_dependencies();
        let querier_wrapper: QuerierWrapper = QuerierWrapper::new(&deps.querier);
        let _: sylvia::types::BoundQuerier<'_, cosmwasm_std::Empty, _> = <SomeContract<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
        > as ContractApi>::Querier::borrowed(
            &owner, &querier_wrapper
        );
    }
}

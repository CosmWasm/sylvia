use cosmwasm_std::{Empty, Response, StdError, StdResult};
use sylvia::types::{CustomMsg, CustomQuery, ExecCtx, InstantiateCtx, QueryCtx, SudoCtx};
use sylvia::{contract, entry_points};

mod interface1 {
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::interface;
    use sylvia::types::{CustomMsg, CustomQuery, ExecCtx, QueryCtx, SudoCtx};

    #[interface]
    pub trait SylviaInterface1 {
        type Error: From<StdError>;
        type ExecC: CustomMsg;
        type QueryC: CustomQuery;

        #[sv::msg(exec)]
        fn interface1_method_exec(
            &self,
            _ctx: ExecCtx<Self::QueryC>,
        ) -> StdResult<Response<Self::ExecC>>;

        #[sv::msg(query)]
        fn interface1_method_query(
            &self,
            _ctx: QueryCtx<Self::QueryC>,
        ) -> StdResult<Response<Self::ExecC>>;

        #[sv::msg(sudo)]
        fn interface1_method_sudo(
            &self,
            _ctx: SudoCtx<Self::QueryC>,
        ) -> StdResult<Response<Self::ExecC>>;
    }
}

mod interface2 {
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::interface;
    use sylvia::types::{CustomMsg, CustomQuery, ExecCtx, QueryCtx, SudoCtx};

    #[interface]
    pub trait SylviaInterface2 {
        type Error: From<StdError>;
        type ExecC: CustomMsg;
        type QueryC: CustomQuery;

        #[sv::msg(exec)]
        fn interface2_method_exec(
            &self,
            _ctx: ExecCtx<Self::QueryC>,
        ) -> StdResult<Response<Self::ExecC>>;

        #[sv::msg(query)]
        fn interface2_method_query(
            &self,
            _ctx: QueryCtx<Self::QueryC>,
        ) -> StdResult<Response<Self::ExecC>>;

        #[sv::msg(sudo)]
        fn interface2_method_sudo(
            &self,
            _ctx: SudoCtx<Self::QueryC>,
        ) -> StdResult<Response<Self::ExecC>>;
    }
}

pub struct Contract<E, Q> {
    _phantom: std::marker::PhantomData<(E, Q)>,
}

// Check that the macro expansion won't fail due to deprecated `, custom` parameter.
#[entry_points(generics<Empty, Empty>, custom(msg=Empty, query=Empty))]
#[contract]
#[sv::messages(interface1)]
#[sv::messages(interface2)]
#[sv::custom(msg=E, query=Q)]
impl<E, Q> Contract<E, Q>
where
    E: CustomMsg + 'static,
    Q: CustomQuery + 'static,
{
    pub fn new() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }

    #[sv::msg(instantiate)]
    fn instantiate(&self, _ctx: InstantiateCtx<Q>) -> StdResult<Response<E>> {
        Ok(Response::new())
    }

    #[sv::msg(exec)]
    fn contract_method_exec(&self, _ctx: ExecCtx<Q>) -> StdResult<Response<E>> {
        Ok(Response::new())
    }

    #[sv::msg(query)]
    fn contract_method_query(&self, _ctx: QueryCtx<Q>) -> StdResult<Response<E>> {
        Ok(Response::new())
    }

    #[sv::msg(sudo)]
    fn contract_method_sudo(&self, _ctx: SudoCtx<Q>) -> StdResult<Response<E>> {
        Ok(Response::new())
    }
}

impl<E, Q> interface1::SylviaInterface1 for Contract<E, Q>
where
    E: CustomMsg + 'static,
    Q: CustomQuery + 'static,
{
    type Error = StdError;
    type ExecC = E;
    type QueryC = Q;

    fn interface1_method_exec(&self, _ctx: ExecCtx<Q>) -> StdResult<Response<E>> {
        Ok(Response::new())
    }

    fn interface1_method_query(&self, _ctx: QueryCtx<Q>) -> StdResult<Response<E>> {
        Ok(Response::new())
    }

    fn interface1_method_sudo(&self, _ctx: SudoCtx<Q>) -> StdResult<Response<E>> {
        Ok(Response::new())
    }
}

impl<E, Q> interface2::SylviaInterface2 for Contract<E, Q>
where
    E: CustomMsg + 'static,
    Q: CustomQuery + 'static,
{
    type Error = StdError;
    type ExecC = E;
    type QueryC = Q;

    fn interface2_method_exec(&self, _ctx: ExecCtx<Q>) -> StdResult<Response<E>> {
        Ok(Response::new())
    }

    fn interface2_method_query(&self, _ctx: QueryCtx<Q>) -> StdResult<Response<E>> {
        Ok(Response::new())
    }

    fn interface2_method_sudo(&self, _ctx: SudoCtx<Q>) -> StdResult<Response<E>> {
        Ok(Response::new())
    }
}

#[test]
fn check_from_trait_implementations() {
    let _ = sv::ContractExecMsg::<Empty, Empty>::from(
        interface1::sv::SylviaInterface1ExecMsg::Interface1MethodExec {},
    );
    let _ = sv::ContractQueryMsg::<Empty, Empty>::from(
        interface1::sv::SylviaInterface1QueryMsg::Interface1MethodQuery {},
    );
    let _ = sv::ContractSudoMsg::<Empty, Empty>::from(
        interface1::sv::SylviaInterface1SudoMsg::Interface1MethodSudo {},
    );

    let _ = sv::ContractExecMsg::<Empty, Empty>::from(
        interface2::sv::SylviaInterface2ExecMsg::Interface2MethodExec {},
    );
    let _ = sv::ContractQueryMsg::<Empty, Empty>::from(
        interface2::sv::SylviaInterface2QueryMsg::Interface2MethodQuery {},
    );
    let _ = sv::ContractSudoMsg::<Empty, Empty>::from(
        interface2::sv::SylviaInterface2SudoMsg::Interface2MethodSudo {},
    );

    let _ = sv::ContractExecMsg::<Empty, Empty>::from(sv::ExecMsg::ContractMethodExec {});
    let _ = sv::ContractQueryMsg::<Empty, Empty>::from(sv::QueryMsg::ContractMethodQuery {});
    let _ = sv::ContractSudoMsg::<Empty, Empty>::from(sv::SudoMsg::ContractMethodSudo {});
}

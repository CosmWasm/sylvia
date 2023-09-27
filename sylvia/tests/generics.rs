use cosmwasm_schema::cw_serde;

pub mod cw1 {
    use cosmwasm_std::{CosmosMsg, CustomMsg, CustomQuery, Response, StdError};

    use serde::{de::DeserializeOwned, Deserialize};
    use sylvia::types::{ExecCtx, QueryCtx};
    use sylvia_derive::interface;

    #[interface(module=msg)]
    #[sv::custom(msg=Msg)]
    pub trait Cw1<Msg, Param, QueryRet>
    where
        for<'msg_de> Msg: CustomMsg + Deserialize<'msg_de>,
        Param: sylvia::types::CustomMsg,
        for<'msg_de> QueryRet: CustomQuery + DeserializeOwned,
    {
        type Error: From<StdError>;

        #[msg(exec)]
        fn execute(
            &self,
            ctx: ExecCtx,
            msgs: Vec<CosmosMsg<Msg>>,
        ) -> Result<Response<Msg>, Self::Error>;

        #[msg(query)]
        fn some_query(&self, ctx: QueryCtx, param: Param) -> Result<QueryRet, Self::Error>;
    }
}

pub mod cw1_contract {
    use cosmwasm_std::{Response, StdResult};
    use sylvia::types::InstantiateCtx;
    use sylvia_derive::contract;

    pub struct Cw1Contract;

    #[contract]
    impl Cw1Contract {
        pub const fn new() -> Self {
            Self
        }

        #[msg(instantiate)]
        pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
            Ok(Response::new())
        }
    }
}

pub mod impl_cw1 {
    use cosmwasm_std::{CosmosMsg, Response, StdError};
    use sylvia::types::{ExecCtx, QueryCtx};
    use sylvia_derive::contract;

    use crate::{cw1::Cw1, cw1_contract::Cw1Contract, ExternalMsg};

    #[contract(module = crate::cw1_contract)]
    #[messages(crate::cw1 as Cw1)]
    impl Cw1<ExternalMsg, crate::ExternalMsg, crate::ExternalQuery> for Cw1Contract {
        type Error = StdError;

        #[msg(exec)]
        fn execute(
            &self,
            _ctx: ExecCtx,
            _msgs: Vec<CosmosMsg<ExternalMsg>>,
        ) -> Result<Response<ExternalMsg>, Self::Error> {
            Ok(Response::new())
        }

        #[msg(query)]
        fn some_query(
            &self,
            _ctx: QueryCtx,
            _param: crate::ExternalMsg,
        ) -> Result<crate::ExternalQuery, Self::Error> {
            Ok(crate::ExternalQuery {})
        }
    }
}

#[cw_serde]
pub struct ExternalMsg;
impl cosmwasm_std::CustomMsg for ExternalMsg {}

#[cw_serde]
pub struct ExternalQuery;
impl cosmwasm_std::CustomQuery for ExternalQuery {}

#[cfg(all(test, feature = "mt"))]
mod tests {
    use cosmwasm_std::{testing::mock_dependencies, Addr, CosmosMsg, Empty, QuerierWrapper};

    use crate::{cw1::Querier, ExternalMsg, ExternalQuery};

    use crate::cw1::InterfaceTypes;
    use sylvia::types::InterfaceMessages;
    #[test]
    fn construct_messages() {
        let contract = Addr::unchecked("contract");

        let _ = crate::cw1::QueryMsg::<_, Empty>::some_query(ExternalMsg {});
        let _ = crate::cw1::ExecMsg::execute(vec![CosmosMsg::Custom(ExternalMsg {})]);
        let _ = crate::cw1::ExecMsg::execute(vec![CosmosMsg::Custom(Empty {})]);

        // Generic Querier
        let deps = mock_dependencies();
        let querier: QuerierWrapper<ExternalQuery> = QuerierWrapper::new(&deps.querier);

        let cw1_querier = crate::cw1::BoundQuerier::borrowed(&contract, &querier);
        let _: Result<ExternalQuery, _> = Querier::some_query(&cw1_querier, ExternalMsg {});
        let _: Result<ExternalQuery, _> = cw1_querier.some_query(ExternalMsg {});

        // Construct messages with Interface extension
        let _ =
            <InterfaceTypes<ExternalMsg, _, ExternalQuery> as InterfaceMessages>::Query::some_query(
                ExternalMsg {},
            );
        let _=
            <InterfaceTypes<_, ExternalMsg, cosmwasm_std::Empty> as InterfaceMessages>::Exec::execute(vec![
                CosmosMsg::Custom(ExternalMsg {}),
            ]);
    }
}

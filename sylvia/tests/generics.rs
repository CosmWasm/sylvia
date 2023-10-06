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
        QueryRet: CustomQuery + DeserializeOwned,
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

pub mod whitelist {
    use cosmwasm_std::{CosmosMsg, CustomMsg, CustomQuery, Response, StdError};

    use serde::de::DeserializeOwned;
    use serde::Deserialize;
    use sylvia::types::{ExecCtx, QueryCtx};
    use sylvia_derive::interface;

    #[interface(module=msg)]
    pub trait Whitelist<Msg, QueryRet>
    where
        for<'msg_de> Msg: CustomMsg + Deserialize<'msg_de>,
        for<'msg_de> QueryRet: CustomQuery + DeserializeOwned,
    {
        type Error: From<StdError>;

        #[msg(exec)]
        fn update_admins(
            &self,
            ctx: ExecCtx,
            msgs: Vec<CosmosMsg<Msg>>,
        ) -> Result<Response, Self::Error>;

        #[msg(query)]
        fn admins_list(&self, ctx: QueryCtx) -> Result<QueryRet, Self::Error>;
    }
}

pub mod non_generic {
    use cosmwasm_std::{CosmosMsg, Empty, Response, StdError};

    use sylvia::types::{ExecCtx, QueryCtx};
    use sylvia_derive::interface;

    #[interface(module=msg)]
    pub trait NonGeneric {
        type Error: From<StdError>;

        #[msg(exec)]
        fn non_generic_exec(
            &self,
            ctx: ExecCtx,
            msgs: Vec<CosmosMsg<Empty>>,
        ) -> Result<Response, Self::Error>;

        #[msg(query)]
        fn non_generic_query(&self, ctx: QueryCtx) -> Result<Response, Self::Error>;
    }
}

pub mod cw1_contract {
    use cosmwasm_std::{Response, StdResult};
    use sylvia::types::InstantiateCtx;
    use sylvia_derive::contract;

    use crate::{ExternalMsg, ExternalQuery};

    pub struct Cw1Contract;

    #[contract]
    #[messages(crate::cw1<ExternalMsg, ExternalMsg, ExternalQuery> as Cw1)]
    #[messages(crate::whitelist<ExternalMsg, ExternalQuery> as Whitelist: custom(msg))]
    #[messages(crate::non_generic as NonGeneric: custom(msg))]
    /// Required if interface returns generic `Response`
    #[sv::custom(msg=ExternalMsg)]
    impl Cw1Contract {
        pub const fn new() -> Self {
            Self
        }

        #[msg(instantiate)]
        pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response<ExternalMsg>> {
            Ok(Response::new())
        }
    }
}

pub mod impl_non_generic {
    use crate::cw1_contract::Cw1Contract;
    use crate::non_generic::NonGeneric;
    use cosmwasm_std::{CosmosMsg, Empty, Response, StdError};
    use sylvia::types::{ExecCtx, QueryCtx};
    use sylvia_derive::contract;

    #[contract(module = crate::cw1_contract)]
    #[messages(crate::non_generic as NonGeneric)]
    #[sv::custom(msg=crate::ExternalMsg)]
    impl NonGeneric for Cw1Contract {
        type Error = StdError;

        #[msg(exec)]
        fn non_generic_exec(
            &self,
            _ctx: ExecCtx,
            _msgs: Vec<CosmosMsg<Empty>>,
        ) -> Result<Response, Self::Error> {
            Ok(Response::new())
        }

        #[msg(query)]
        fn non_generic_query(&self, _ctx: QueryCtx) -> Result<Response, Self::Error> {
            Ok(Response::default())
        }
    }
}

pub mod impl_whitelist {
    use cosmwasm_std::{CosmosMsg, Response, StdError};
    use sylvia::types::{ExecCtx, QueryCtx};
    use sylvia_derive::contract;

    use crate::cw1_contract::Cw1Contract;
    use crate::whitelist::Whitelist;
    use crate::{ExternalMsg, ExternalQuery};

    #[contract(module = crate::cw1_contract)]
    #[messages(crate::whitelist as Whitelist)]
    #[sv::custom(msg=ExternalMsg)]
    impl Whitelist<ExternalMsg, ExternalQuery> for Cw1Contract {
        type Error = StdError;

        #[msg(exec)]
        fn update_admins(
            &self,
            _ctx: ExecCtx,
            _msgs: Vec<CosmosMsg<ExternalMsg>>,
        ) -> Result<Response, Self::Error> {
            Ok(Response::new())
        }

        #[msg(query)]
        fn admins_list(&self, _ctx: QueryCtx) -> Result<ExternalQuery, Self::Error> {
            Ok(ExternalQuery {})
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
    #[sv::custom(msg=ExternalMsg)]
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
    use crate::cw1::{InterfaceTypes, Querier as Cw1Querier};
    use crate::cw1_contract::Cw1Contract;
    use crate::impl_cw1::test_utils::Cw1;
    use crate::impl_non_generic::test_utils::NonGeneric;
    use crate::impl_whitelist::test_utils::Whitelist;
    use crate::non_generic::Querier as NonGenericQuerier;
    use crate::whitelist::Querier as WhitelistQuerier;
    use crate::{ExternalMsg, ExternalQuery};
    use cosmwasm_std::{testing::mock_dependencies, Addr, CosmosMsg, Empty, QuerierWrapper};
    use sylvia::multitest::App;
    use sylvia::types::InterfaceMessages;

    #[test]
    fn construct_messages() {
        let contract = Addr::unchecked("contract");

        // Direct message construction
        // cw1
        let _ = crate::cw1::QueryMsg::<_, Empty>::some_query(ExternalMsg {});
        let _ = crate::cw1::ExecMsg::execute(vec![CosmosMsg::Custom(ExternalMsg {})]);
        let _ = crate::cw1::ExecMsg::execute(vec![CosmosMsg::Custom(Empty {})]);

        // whitelist
        let _ = crate::whitelist::QueryMsg::<ExternalQuery>::admins_list();
        let _ = crate::whitelist::ExecMsg::update_admins(vec![CosmosMsg::Custom(ExternalMsg {})]);

        // non_generic
        let _ = crate::non_generic::QueryMsg::non_generic_query();
        let _ = crate::non_generic::ExecMsg::non_generic_exec(vec![]);

        // Generic Querier
        let deps = mock_dependencies();
        let querier: QuerierWrapper<ExternalQuery> = QuerierWrapper::new(&deps.querier);

        let cw1_querier = crate::cw1::BoundQuerier::borrowed(&contract, &querier);
        let _: Result<ExternalQuery, _> =
            crate::cw1::Querier::some_query(&cw1_querier, ExternalMsg {});
        let _: Result<ExternalQuery, _> = cw1_querier.some_query(ExternalMsg {});

        let contract_querier = crate::cw1_contract::BoundQuerier::borrowed(&contract, &querier);
        let _: Result<ExternalQuery, _> = contract_querier.some_query(ExternalMsg {});
        let _: Result<ExternalQuery, _> = contract_querier.admins_list();
        let _ = contract_querier.non_generic_query();

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

    #[test]
    fn mt_helpers() {
        let _ = Cw1Contract::new();
        let app = App::<cw_multi_test::BasicApp<ExternalMsg>>::custom(|_, _, _| {});
        let code_id = crate::cw1_contract::multitest_utils::CodeId::store_code(&app);

        let owner = "owner";

        let contract = code_id
            .instantiate()
            .with_label("Cw1Contract")
            .call(owner)
            .unwrap();

        // CustomMsg generic Interface
        contract.cw1_proxy().some_query(ExternalMsg {}).unwrap();
        contract.cw1_proxy().execute(vec![]).call(owner).unwrap();

        // Non-CustomMsg generic Interface
        contract.whitelist_proxy().admins_list().unwrap();
        contract
            .whitelist_proxy()
            .update_admins(vec![])
            .call(owner)
            .unwrap();

        // Non-CustomMsg non-generic Interface
        contract.non_generic_proxy().non_generic_query().unwrap();
        contract
            .non_generic_proxy()
            .non_generic_exec(vec![])
            .call(owner)
            .unwrap();
    }
}

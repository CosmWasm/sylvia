use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CustomQuery, Response, StdResult};
use sylvia::contract;
use sylvia::types::{ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx};

#[cw_serde]
pub struct MyQuery;

impl CustomQuery for MyQuery {}

#[cw_serde]
pub struct OtherQuery;

impl CustomQuery for OtherQuery {}

pub struct MyContract;

#[cw_serde]
pub struct SomeResponse;

mod interface {
    use cosmwasm_std::{CustomQuery, Response, StdError, StdResult};
    use sylvia::types::{ExecCtx, QueryCtx};
    use sylvia::{contract, interface};

    use crate::{MyQuery, OtherQuery, SomeResponse};

    #[interface]
    #[sv::custom(query=MyQuery)]
    pub trait Interface {
        type Error: From<StdError>;
        type QueryC: CustomQuery;

        #[cfg(not(tarpaulin_include))]
        #[msg(query)]
        fn interface_query(&self, ctx: QueryCtx<MyQuery>) -> StdResult<SomeResponse>;

        #[cfg(not(tarpaulin_include))]
        #[msg(exec)]
        fn interface_exec(&self, ctx: ExecCtx<MyQuery>) -> StdResult<Response>;
    }

    #[contract(module=super)]
    #[sv::custom(query=MyQuery)]
    impl Interface for crate::MyContract {
        type Error = StdError;
        type QueryC = OtherQuery;

        #[msg(query)]
        fn interface_query(&self, _ctx: QueryCtx<MyQuery>) -> StdResult<SomeResponse> {
            Ok(SomeResponse)
        }

        #[msg(exec)]
        fn interface_exec(&self, _ctx: ExecCtx<MyQuery>) -> StdResult<Response> {
            Ok(Response::default())
        }
    }
}

mod some_interface {
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::types::{ExecCtx, QueryCtx};
    use sylvia::{contract, interface};

    use crate::{MyQuery, SomeResponse};

    #[interface]
    #[sv::custom(query=MyQuery)]
    pub trait SomeInterface {
        type Error: From<StdError>;

        #[cfg(not(tarpaulin_include))]
        #[msg(query)]
        fn some_interface_query(&self, ctx: QueryCtx<MyQuery>) -> StdResult<SomeResponse>;

        #[cfg(not(tarpaulin_include))]
        #[msg(exec)]
        fn some_interface_exec(&self, ctx: ExecCtx<MyQuery>) -> StdResult<Response>;
    }

    #[contract(module=super)]
    #[sv::custom(query=MyQuery)]
    impl SomeInterface for crate::MyContract {
        type Error = StdError;

        #[msg(query)]
        fn some_interface_query(&self, _ctx: QueryCtx<MyQuery>) -> StdResult<SomeResponse> {
            Ok(SomeResponse)
        }

        #[msg(exec)]
        fn some_interface_exec(&self, _ctx: ExecCtx<MyQuery>) -> StdResult<Response> {
            Ok(Response::default())
        }
    }
}

mod associated_type_interface {
    use cosmwasm_std::{CustomQuery, Response, StdError, StdResult};
    use sylvia::types::{ExecCtx, QueryCtx};
    use sylvia::{contract, interface};

    use crate::{MyQuery, SomeResponse};

    #[interface]
    pub trait AssociatedTypeInterface {
        type Error: From<StdError>;
        type QueryC: CustomQuery;

        #[cfg(not(tarpaulin_include))]
        #[msg(query)]
        fn associated_query(&self, ctx: QueryCtx<Self::QueryC>) -> StdResult<SomeResponse>;

        #[cfg(not(tarpaulin_include))]
        #[msg(exec)]
        fn associated_exec(&self, ctx: ExecCtx<Self::QueryC>) -> StdResult<Response>;
    }

    #[contract(module=super)]
    impl AssociatedTypeInterface for crate::MyContract {
        type Error = StdError;
        type QueryC = MyQuery;

        #[msg(query)]
        fn associated_query(&self, _ctx: QueryCtx<Self::QueryC>) -> StdResult<SomeResponse> {
            Ok(SomeResponse)
        }

        #[msg(exec)]
        fn associated_exec(&self, _ctx: ExecCtx<Self::QueryC>) -> StdResult<Response> {
            Ok(Response::default())
        }
    }
}

mod default_query_interface {
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::types::{ExecCtx, QueryCtx};
    use sylvia::{contract, interface};

    use crate::SomeResponse;

    #[interface]
    pub trait DefaultQueryInterface {
        type Error: From<StdError>;

        #[cfg(not(tarpaulin_include))]
        #[msg(query)]
        fn default_query(&self, ctx: QueryCtx) -> StdResult<SomeResponse>;

        #[cfg(not(tarpaulin_include))]
        #[msg(exec)]
        fn default_exec(&self, ctx: ExecCtx) -> StdResult<Response>;
    }

    #[contract(module=super)]
    #[sv::custom(query=MyQuery)]
    impl DefaultQueryInterface for crate::MyContract {
        type Error = StdError;

        #[msg(query)]
        fn default_query(&self, _ctx: QueryCtx) -> StdResult<SomeResponse> {
            Ok(SomeResponse)
        }

        #[msg(exec)]
        fn default_exec(&self, _ctx: ExecCtx) -> StdResult<Response> {
            Ok(Response::default())
        }
    }
}

#[contract]
#[messages(some_interface as SomeInterface)]
#[messages(associated_type_interface as AssociatedTypeInterface)]
#[messages(interface as Interface)]
#[messages(default_query_interface as DefaultQueryInterface: custom(query))]
#[sv::custom(query=MyQuery)]
impl MyContract {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    #[msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx<MyQuery>) -> StdResult<Response> {
        Ok(Response::default())
    }

    #[msg(exec)]
    pub fn some_exec(&self, _ctx: ExecCtx<MyQuery>) -> StdResult<Response> {
        Ok(Response::default())
    }

    #[msg(query)]
    pub fn some_query(&self, _ctx: QueryCtx<MyQuery>) -> StdResult<SomeResponse> {
        Ok(SomeResponse)
    }

    #[cfg(not(tarpaulin_include))]
    #[msg(migrate)]
    pub fn some_migrate(&self, _ctx: MigrateCtx<MyQuery>) -> StdResult<Response> {
        Ok(Response::default())
    }
}

#[cfg(all(test, feature = "mt"))]
mod tests {
    use crate::associated_type_interface::test_utils::AssociatedTypeInterface;
    use crate::default_query_interface::test_utils::DefaultQueryInterface;
    use crate::some_interface::test_utils::SomeInterface;
    use crate::{interface::test_utils::Interface, MyContract, MyQuery};

    use cosmwasm_std::Empty;
    use sylvia::multitest::App;

    #[test]
    fn test_custom() {
        let _ = MyContract::new();
        let app = App::<cw_multi_test::BasicApp<Empty, MyQuery>>::custom(|_, _, _| {});
        let code_id = crate::multitest_utils::CodeId::store_code(&app);

        let owner = "owner";

        let contract = code_id
            .instantiate()
            .with_label("MyContract")
            .call(owner)
            .unwrap();

        contract.some_exec().call(owner).unwrap();
        contract.some_query().unwrap();

        // `sv::custom` attribute interface
        contract
            .some_interface_proxy()
            .some_interface_query()
            .unwrap();
        contract
            .some_interface_proxy()
            .some_interface_exec()
            .call(owner)
            .unwrap();

        // Associated tyoe interface messages
        contract
            .associated_type_interface_proxy()
            .associated_query()
            .unwrap();
        contract
            .associated_type_interface_proxy()
            .associated_exec()
            .call(owner)
            .unwrap();

        // `sv::custom` attribute and associated type interface
        contract.interface_proxy().interface_query().unwrap();
        contract
            .interface_proxy()
            .interface_exec()
            .call(owner)
            .unwrap();

        // Neither `custom` attribute nor associated type
        contract
            .default_query_interface_proxy()
            .default_query()
            .unwrap();
        contract
            .default_query_interface_proxy()
            .default_exec()
            .call(owner)
            .unwrap();
    }
}

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CustomQuery, Response, StdResult};
use sylvia::contract;
use sylvia::types::{ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx, SudoCtx};

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
    use sylvia::interface;
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};

    use crate::{MyQuery, SomeResponse};

    #[interface]
    #[sv::custom(query=MyQuery)]
    pub trait Interface {
        type Error: From<StdError>;
        type QueryC: CustomQuery;

        #[sv::msg(query)]
        fn interface_query(&self, ctx: QueryCtx<MyQuery>) -> StdResult<SomeResponse>;

        #[sv::msg(exec)]
        fn interface_exec(&self, ctx: ExecCtx<MyQuery>) -> StdResult<Response>;

        #[sv::msg(sudo)]
        fn interface_sudo(&self, ctx: SudoCtx<MyQuery>) -> StdResult<Response>;
    }
}

mod impl_interface {
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};
    use sylvia_derive::contract;

    use crate::{MyQuery, OtherQuery, SomeResponse};

    #[contract(module=crate)]
    #[sv::messages(crate::interface)]
    #[sv::custom(query=MyQuery)]
    impl crate::interface::Interface for crate::MyContract {
        type Error = StdError;
        type QueryC = OtherQuery;

        #[sv::msg(query)]
        fn interface_query(&self, _ctx: QueryCtx<MyQuery>) -> StdResult<SomeResponse> {
            Ok(SomeResponse)
        }

        #[sv::msg(exec)]
        fn interface_exec(&self, _ctx: ExecCtx<MyQuery>) -> StdResult<Response> {
            Ok(Response::default())
        }

        #[sv::msg(sudo)]
        fn interface_sudo(&self, _ctx: SudoCtx<MyQuery>) -> StdResult<Response> {
            Ok(Response::default())
        }
    }
}

mod some_interface {
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::interface;
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};

    use crate::{MyQuery, SomeResponse};

    #[interface]
    #[sv::custom(query=MyQuery)]
    pub trait SomeInterface {
        type Error: From<StdError>;

        #[sv::msg(query)]
        fn some_interface_query(&self, ctx: QueryCtx<MyQuery>) -> StdResult<SomeResponse>;

        #[sv::msg(exec)]
        fn some_interface_exec(&self, ctx: ExecCtx<MyQuery>) -> StdResult<Response>;

        #[sv::msg(sudo)]
        fn some_interface_sudo(&self, ctx: SudoCtx<MyQuery>) -> StdResult<Response>;
    }
}

mod impl_some_interface {
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};
    use sylvia_derive::contract;

    use crate::{MyQuery, SomeResponse};

    #[contract(module=crate)]
    #[sv::messages(crate::some_interface)]
    #[sv::custom(query=MyQuery)]
    impl super::some_interface::SomeInterface for crate::MyContract {
        type Error = StdError;

        #[sv::msg(query)]
        fn some_interface_query(&self, _ctx: QueryCtx<MyQuery>) -> StdResult<SomeResponse> {
            Ok(SomeResponse)
        }

        #[sv::msg(exec)]
        fn some_interface_exec(&self, _ctx: ExecCtx<MyQuery>) -> StdResult<Response> {
            Ok(Response::default())
        }

        #[sv::msg(sudo)]
        fn some_interface_sudo(&self, _ctx: SudoCtx<MyQuery>) -> StdResult<Response> {
            Ok(Response::default())
        }
    }
}

mod associated_type_interface {
    use cosmwasm_std::{CustomQuery, Response, StdError, StdResult};
    use sylvia::interface;
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};

    use crate::SomeResponse;

    #[interface]
    pub trait AssociatedTypeInterface {
        type Error: From<StdError>;
        type QueryC: CustomQuery;

        #[sv::msg(query)]
        fn associated_query(&self, ctx: QueryCtx<Self::QueryC>) -> StdResult<SomeResponse>;

        #[sv::msg(exec)]
        fn associated_exec(&self, ctx: ExecCtx<Self::QueryC>) -> StdResult<Response>;

        #[sv::msg(sudo)]
        fn associated_sudo(&self, ctx: SudoCtx<Self::QueryC>) -> StdResult<Response>;
    }
}

mod impl_associated_type_interface {
    use crate::associated_type_interface::AssociatedTypeInterface;
    use crate::{MyQuery, SomeResponse};
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};
    use sylvia_derive::contract;

    #[contract(module=crate)]
    #[sv::messages(crate::associated_type_interface)]
    impl AssociatedTypeInterface for crate::MyContract {
        type Error = StdError;
        type QueryC = MyQuery;

        #[sv::msg(query)]
        fn associated_query(&self, _ctx: QueryCtx<Self::QueryC>) -> StdResult<SomeResponse> {
            Ok(SomeResponse)
        }

        #[sv::msg(exec)]
        fn associated_exec(&self, _ctx: ExecCtx<Self::QueryC>) -> StdResult<Response> {
            Ok(Response::default())
        }

        #[sv::msg(sudo)]
        fn associated_sudo(&self, _ctx: SudoCtx<Self::QueryC>) -> StdResult<Response> {
            Ok(Response::default())
        }
    }
}

mod default_query_interface {
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::interface;
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};

    use crate::SomeResponse;

    #[interface]
    pub trait DefaultQueryInterface {
        type Error: From<StdError>;

        #[sv::msg(query)]
        fn default_query(&self, ctx: QueryCtx) -> StdResult<SomeResponse>;

        #[sv::msg(exec)]
        fn default_exec(&self, ctx: ExecCtx) -> StdResult<Response>;

        #[sv::msg(sudo)]
        fn default_sudo(&self, ctx: SudoCtx) -> StdResult<Response>;
    }
}

mod impl_default_query_interface {
    use crate::default_query_interface::DefaultQueryInterface;
    use crate::SomeResponse;
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};
    use sylvia_derive::contract;

    #[contract(module=crate)]
    #[sv::messages(crate::default_query_interface)]
    #[sv::custom(query=MyQuery)]
    impl DefaultQueryInterface for crate::MyContract {
        type Error = StdError;

        #[sv::msg(query)]
        fn default_query(&self, _ctx: QueryCtx) -> StdResult<SomeResponse> {
            Ok(SomeResponse)
        }

        #[sv::msg(exec)]
        fn default_exec(&self, _ctx: ExecCtx) -> StdResult<Response> {
            Ok(Response::default())
        }

        #[sv::msg(sudo)]
        fn default_sudo(&self, _ctx: SudoCtx) -> StdResult<Response> {
            Ok(Response::default())
        }
    }
}

#[contract]
#[sv::messages(some_interface)]
#[sv::messages(associated_type_interface)]
#[sv::messages(interface)]
#[sv::messages(default_query_interface: custom(query))]
#[sv::custom(query=MyQuery)]
impl MyContract {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx<MyQuery>) -> StdResult<Response> {
        Ok(Response::default())
    }

    #[sv::msg(exec)]
    pub fn some_exec(&self, _ctx: ExecCtx<MyQuery>) -> StdResult<Response> {
        Ok(Response::default())
    }

    #[sv::msg(query)]
    pub fn some_query(&self, _ctx: QueryCtx<MyQuery>) -> StdResult<SomeResponse> {
        Ok(SomeResponse)
    }

    #[sv::msg(migrate)]
    pub fn some_migrate(&self, _ctx: MigrateCtx<MyQuery>) -> StdResult<Response> {
        Ok(Response::default())
    }

    #[sv::msg(sudo)]
    pub fn some_sudo(&self, _ctx: SudoCtx<MyQuery>) -> StdResult<Response> {
        Ok(Response::default())
    }
}

#[cfg(all(test, feature = "mt"))]
mod tests {
    use crate::impl_associated_type_interface::sv::test_utils::AssociatedTypeInterface;
    use crate::impl_default_query_interface::sv::test_utils::DefaultQueryInterface;
    use crate::impl_interface::sv::test_utils::Interface;
    use crate::impl_some_interface::sv::test_utils::SomeInterface;
    use crate::{MyContract, MyQuery};

    use cosmwasm_std::Empty;
    use sylvia::multitest::App;

    #[test]
    fn test_custom() {
        let _ = MyContract::new();
        let app = App::<cw_multi_test::BasicApp<Empty, MyQuery>>::custom(|_, _, _| {});
        let code_id = crate::sv::multitest_utils::CodeId::store_code(&app);

        let owner = "owner";

        let contract = code_id
            .instantiate()
            .with_label("MyContract")
            .with_admin(owner)
            .call(owner)
            .unwrap();

        contract.some_exec().call(owner).unwrap();
        contract.some_query().unwrap();
        contract.some_sudo().unwrap();
        contract
            .some_migrate()
            .call(owner, code_id.code_id())
            .unwrap();

        // `sv::custom` attribute interface
        contract.some_interface_query().unwrap();
        contract.some_interface_exec().call(owner).unwrap();
        contract.some_interface_sudo().unwrap();

        // Associated tyoe interface messages
        contract.associated_query().unwrap();
        contract.associated_exec().call(owner).unwrap();
        contract.associated_sudo().unwrap();

        // `sv::custom` attribute and associated type interface
        contract.interface_query().unwrap();
        contract.interface_exec().call(owner).unwrap();
        contract.interface_sudo().unwrap();

        // Neither `custom` attribute nor associated type
        contract.default_query().unwrap();
        contract.default_exec().call(owner).unwrap();
        contract.default_sudo().unwrap();
    }
}

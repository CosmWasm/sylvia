use cosmwasm_std::{CustomMsg, Response, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sylvia::types::{ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx, SudoCtx};
use sylvia::{contract, entry_points};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
pub struct MyMsg;

impl CustomMsg for MyMsg {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
pub struct OtherMsg;

impl CustomMsg for OtherMsg {}

pub struct MyContract;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
pub struct SomeResponse;

mod some_interface {
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::interface;
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};

    use crate::{MyMsg, SomeResponse};

    #[interface]
    #[sv::custom(msg=MyMsg, query=cosmwasm_std::Empty)]
    pub trait SomeInterface {
        type Error: From<StdError>;

        #[sv::msg(query)]
        fn interface_query(&self, ctx: QueryCtx) -> StdResult<SomeResponse>;

        #[sv::msg(exec)]
        fn interface_exec(&self, ctx: ExecCtx) -> StdResult<Response<MyMsg>>;

        #[sv::msg(sudo)]
        fn interface_sudo(&self, ctx: SudoCtx) -> StdResult<Response<MyMsg>>;
    }
}

mod impl_some_interface {
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};

    use crate::some_interface::SomeInterface;
    use crate::{MyMsg, SomeResponse};

    impl SomeInterface for crate::MyContract {
        type Error = StdError;

        fn interface_query(&self, _ctx: QueryCtx) -> StdResult<SomeResponse> {
            Ok(SomeResponse)
        }

        fn interface_exec(&self, _ctx: ExecCtx) -> StdResult<Response<MyMsg>> {
            Ok(Response::default())
        }

        fn interface_sudo(&self, _ctx: SudoCtx) -> StdResult<Response<MyMsg>> {
            Ok(Response::new())
        }
    }
}

// Use `#[sv::custom(..)]` if both it and associated type defined
mod interface {
    use crate::MyMsg;
    use cosmwasm_std::{CustomMsg, Response, StdError, StdResult};
    use sylvia::interface;
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};

    #[interface]
    #[sv::custom(msg=MyMsg, query=cosmwasm_std::Empty)]
    pub trait Interface {
        type Error: From<StdError>;
        type ExecC: CustomMsg;

        #[sv::msg(exec)]
        fn exec(&self, ctx: ExecCtx) -> StdResult<Response<MyMsg>>;

        #[sv::msg(query)]
        fn query(&self, ctx: QueryCtx) -> StdResult<Response<MyMsg>>;

        #[sv::msg(sudo)]
        fn sudo(&self, ctx: SudoCtx) -> StdResult<Response<MyMsg>>;
    }
}
mod impl_interface {
    use crate::interface::Interface;
    use crate::{MyMsg, OtherMsg};
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};

    impl Interface for crate::MyContract {
        type Error = StdError;
        type ExecC = OtherMsg;

        fn exec(&self, _ctx: ExecCtx) -> StdResult<Response<MyMsg>> {
            Ok(Response::default())
        }

        fn query(&self, _ctx: QueryCtx) -> StdResult<Response<MyMsg>> {
            Ok(Response::new())
        }

        fn sudo(&self, _ctx: SudoCtx) -> StdResult<Response<MyMsg>> {
            Ok(Response::new())
        }
    }
}

mod other_interface {
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::interface;
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};

    #[interface]
    #[sv::custom(msg=cosmwasm_std::Empty, query=cosmwasm_std::Empty)]
    pub trait OtherInterface {
        type Error: From<StdError>;

        #[sv::msg(exec)]
        fn other_interface_exec(&self, ctx: ExecCtx) -> StdResult<Response>;

        #[sv::msg(sudo)]
        fn other_interface_sudo(&self, ctx: SudoCtx) -> StdResult<Response>;

        #[sv::msg(query)]
        fn other_interface_query(&self, ctx: QueryCtx) -> StdResult<Response>;
    }
}
mod impl_other_interface {
    use crate::other_interface::OtherInterface;
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};

    impl OtherInterface for crate::MyContract {
        type Error = StdError;

        fn other_interface_exec(&self, _ctx: ExecCtx) -> StdResult<Response> {
            Ok(Response::default())
        }

        fn other_interface_sudo(&self, _ctx: SudoCtx) -> StdResult<Response> {
            Ok(Response::new())
        }

        fn other_interface_query(&self, _ctx: QueryCtx) -> StdResult<Response> {
            Ok(Response::new())
        }
    }
}

mod associated_interface {
    use crate::SomeResponse;
    use cosmwasm_std::{CustomMsg, Response, StdError, StdResult};
    use sylvia::interface;
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};

    #[interface]
    #[sv::custom(query=cosmwasm_std::Empty)]
    pub trait AssociatedInterface {
        type Error: From<StdError>;
        type ExecC: CustomMsg;

        #[sv::msg(exec)]
        fn associated_exec(&self, ctx: ExecCtx) -> StdResult<Response<Self::ExecC>>;

        #[sv::msg(sudo)]
        fn associated_sudo(&self, ctx: SudoCtx) -> StdResult<Response<Self::ExecC>>;

        #[sv::msg(query)]
        fn associated_query(&self, ctx: QueryCtx) -> StdResult<SomeResponse>;
    }
}
mod impl_associated_interface {
    use crate::associated_interface::AssociatedInterface;
    use crate::{MyMsg, SomeResponse};
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};

    impl AssociatedInterface for crate::MyContract {
        type Error = StdError;
        type ExecC = MyMsg;

        fn associated_exec(&self, _ctx: ExecCtx) -> StdResult<Response<Self::ExecC>> {
            Ok(Response::default())
        }

        fn associated_sudo(&self, _ctx: SudoCtx) -> StdResult<Response<Self::ExecC>> {
            Ok(Response::default())
        }

        fn associated_query(&self, _ctx: QueryCtx) -> StdResult<SomeResponse> {
            Ok(SomeResponse {})
        }
    }
}

#[entry_points]
#[contract]
#[sv::messages(some_interface)]
#[sv::messages(other_interface: custom(msg))]
#[sv::messages(associated_interface)]
#[sv::messages(interface)]
#[sv::custom(msg=MyMsg)]
impl MyContract {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response<MyMsg>> {
        Ok(Response::default())
    }

    #[sv::msg(exec)]
    pub fn some_exec(&self, _ctx: ExecCtx) -> StdResult<Response<MyMsg>> {
        Ok(Response::default())
    }

    #[sv::msg(query)]
    pub fn some_query(&self, _ctx: QueryCtx) -> StdResult<SomeResponse> {
        Ok(SomeResponse)
    }

    #[sv::msg(migrate)]
    pub fn some_migrate(&self, _ctx: MigrateCtx) -> StdResult<Response<MyMsg>> {
        Ok(Response::default())
    }

    #[sv::msg(sudo)]
    pub fn some_sudo(&self, _ctx: SudoCtx) -> StdResult<Response<MyMsg>> {
        Ok(Response::default())
    }
}

#[cfg(all(test, feature = "mt"))]
mod tests {
    use crate::associated_interface::sv::mt::AssociatedInterfaceProxy;
    use crate::interface::sv::mt::InterfaceProxy;
    use crate::other_interface::sv::mt::OtherInterfaceProxy;
    use crate::some_interface::sv::mt::SomeInterfaceProxy;
    use crate::sv::mt::MyContractProxy;
    use crate::{MyContract, MyMsg};
    use cosmwasm_std::Addr;
    use cw_multi_test::IntoBech32;
    use sylvia::multitest::App;

    #[test]
    fn test_custom() {
        let _ = MyContract::new();
        let app = App::<cw_multi_test::BasicApp<MyMsg>>::custom(|_, _, _| {});
        let code_id = crate::sv::mt::CodeId::store_code(&app);

        let owner = "owner".into_bech32();
        let admin = Addr::unchecked("admin");

        let contract = code_id
            .instantiate()
            .with_label("MyContract")
            .with_admin(admin.as_str())
            .call(&owner)
            .unwrap();

        contract.some_exec().call(&owner).unwrap();
        contract.some_query().unwrap();
        contract.some_sudo().unwrap();
        contract
            .some_migrate()
            .call(&admin, code_id.code_id())
            .unwrap();

        // Interface messsages
        contract.interface_query().unwrap();
        contract.interface_exec().call(&owner).unwrap();
        contract.interface_sudo().unwrap();

        // Other interface messages
        contract.other_interface_query().unwrap();
        contract.other_interface_exec().call(&owner).unwrap();
        contract.other_interface_sudo().unwrap();

        // Associated interface messages
        contract.associated_query().unwrap();
        contract.associated_exec().call(&owner).unwrap();
        contract.associated_sudo().unwrap();

        // Both associated type and custom attr used
        contract.query().unwrap();
        contract.exec().call(&owner).unwrap();
        contract.sudo().unwrap();
    }
}

use cosmwasm_std::{CustomMsg, CustomQuery, Response, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sylvia::contract;
use sylvia::types::{ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
pub struct MyMsg;

impl CustomMsg for MyMsg {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
pub struct MyQuery;

impl CustomQuery for MyQuery {}

pub struct MyContract;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
pub struct SomeResponse;

mod some_interface {
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::types::{ExecCtx, QueryCtx};
    use sylvia::{contract, interface};

    use crate::{MyMsg, SomeResponse};

    #[interface]
    #[sv::custom(msg=MyMsg, query=MyQuery)]
    pub trait SomeInterface {
        type Error: From<StdError>;

        #[cfg(not(tarpaulin_include))]
        #[msg(query)]
        fn interface_query(&self, ctx: QueryCtx) -> StdResult<SomeResponse>;

        #[cfg(not(tarpaulin_include))]
        #[msg(exec)]
        fn interface_exec(&self, ctx: ExecCtx) -> StdResult<Response<MyMsg>>;
    }

    #[contract(module=super)]
    #[sv::custom(msg=MyMsg, query=MyQuery)]
    impl SomeInterface for crate::MyContract {
        type Error = StdError;

        #[msg(query)]
        fn interface_query(&self, _ctx: QueryCtx) -> StdResult<SomeResponse> {
            Ok(SomeResponse)
        }

        #[msg(exec)]
        fn interface_exec(&self, _ctx: ExecCtx) -> StdResult<Response<MyMsg>> {
            Ok(Response::default())
        }
    }
}

mod other_interface {
    use crate::MyMsg;
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::types::ExecCtx;
    use sylvia::{contract, interface};

    #[interface]
    pub trait OtherInterface {
        type Error: From<StdError>;

        #[cfg(not(tarpaulin_include))]
        #[msg(exec)]
        fn other_interface_exec(&self, ctx: ExecCtx) -> StdResult<Response>;
    }

    #[contract(module=super)]
    #[sv::custom(msg=MyMsg)]
    impl OtherInterface for crate::MyContract {
        type Error = StdError;

        #[msg(exec)]
        fn other_interface_exec(&self, _ctx: ExecCtx) -> StdResult<Response> {
            Ok(Response::default())
        }
    }
}

#[contract]
#[messages(some_interface as SomeInterface)]
#[messages(other_interface as OtherInterface: custom(msg))]
#[sv::custom(msg=MyMsg, query=MyQuery)]
impl MyContract {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    #[msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response<MyMsg>> {
        Ok(Response::default())
    }

    #[msg(exec)]
    pub fn some_exec(&self, _ctx: ExecCtx) -> StdResult<Response<MyMsg>> {
        Ok(Response::default())
    }

    #[msg(query)]
    pub fn some_query(&self, _ctx: QueryCtx) -> StdResult<SomeResponse> {
        Ok(SomeResponse)
    }

    #[cfg(not(tarpaulin_include))]
    #[msg(migrate)]
    pub fn some_migrate(&self, _ctx: MigrateCtx) -> StdResult<Response<MyMsg>> {
        Ok(Response::default())
    }
}

#[cfg(test)]
mod tests {
    use crate::MyContract;
    use sylvia::multitest::App;

    use crate::other_interface::test_utils::OtherInterface;
    use crate::some_interface::test_utils::SomeInterface;
    use crate::MyMsg;

    #[test]
    fn test_custom() {
        let _ = MyContract::new();
        let app = App::<cw_multi_test::BasicApp<MyMsg>>::custom(|_, _, _| {});
        let code_id = crate::multitest_utils::CodeId::store_code(&app);

        let owner = "owner";

        let contract = code_id
            .instantiate()
            .with_label("MyContract")
            .call(owner)
            .unwrap();

        contract.some_exec().call(owner).unwrap();
        contract.some_query().unwrap();

        // Interface messsages
        contract.some_interface_proxy().interface_query().unwrap();
        contract
            .some_interface_proxy()
            .interface_exec()
            .call(owner)
            .unwrap();

        // Other interface messages
        contract
            .other_interface_proxy()
            .other_interface_exec()
            .call(owner)
            .unwrap();
    }
}

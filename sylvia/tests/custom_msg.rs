use cosmwasm_std::{CustomMsg, CustomQuery, Response, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sylvia::types::{ExecCtx, InstantiateCtx, QueryCtx};
use sylvia::{contract, entry_points};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
pub struct MyMsg;

impl CustomMsg for MyMsg {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
pub struct MyQuery;

impl CustomQuery for MyMsg {}

pub struct MyContract;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
pub struct SomeResponse {}

#[entry_points]
#[contract]
#[sv::custom(msg=MyMsg)]
impl MyContract {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    #[msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response<MyMsg>> {
        Ok(Response::new())
    }

    #[msg(exec)]
    pub fn some_exec(&self, _ctx: ExecCtx) -> StdResult<Response<MyMsg>> {
        Ok(Response::default())
    }

    #[msg(query)]
    pub fn some_query(&self, _ctx: QueryCtx) -> StdResult<SomeResponse> {
        Ok(SomeResponse {})
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Empty;
    use sylvia::multitest::App;

    use crate::MyMsg;

    #[test]
    fn test_custom() {
        let app = App::custom::<MyMsg, Empty, _>(|_, _, _| {});
        let code_id = crate::multitest_utils::CodeId::store_code(&app);

        let owner = "owner";

        let contract = code_id
            .instantiate()
            .with_label("MyContract")
            .call(owner)
            .unwrap();

        contract.some_exec().call(owner).unwrap();
        contract.some_query().unwrap();
    }
}

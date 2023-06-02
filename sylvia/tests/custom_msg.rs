use cosmwasm_std::{CustomMsg, CustomQuery, Response, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sylvia::types::InstantiateCtx;
use sylvia::{contract, entry_points};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
pub struct MyMsg;

impl CustomMsg for MyMsg {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
pub struct MyQuery;

impl CustomQuery for MyMsg {}

pub struct MyContract;

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
        Ok(Response::<MyMsg>::default())
    }

    #[msg(exec)]
    pub fn some_exec(&self, _ctx: InstantiateCtx) -> StdResult<Response<MyMsg>> {
        Ok(Response::<MyMsg>::default())
    }
}

#[cfg(test)]
mod tests {
    use crate::MyContract;

    #[test]
    fn test_custom() {
        let _ = MyContract::new();
    }
}

use cosmwasm_std::{CustomMsg, CustomQuery, Response, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sylvia::contract;
use sylvia::types::InstantiateCtx;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
struct MyMsg;

impl CustomMsg for MyMsg {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
struct MyQuery;

impl CustomQuery for MyMsg {}

pub struct MyContract;

#[contract]
#[sv::custom(msg=MyMsg, query=MyQuery)]
impl MyContract {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    #[msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::default())
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

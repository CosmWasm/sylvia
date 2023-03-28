use cosmwasm_std::{Response, StdError, StdResult};

use sylvia::{
    contract, interface,
    types::{InstantiateCtx, QueryCtx},
};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
}

#[derive(
    serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema,
)]
pub struct QueryResponse;

type QueryResult<E> = Result<QueryResponse, E>;

#[interface(module=msg)]
pub trait Interface {
    type Error: From<StdError>;

    #[msg(query, resp=QueryResponse)]
    fn query(&self, ctx: QueryCtx, #[serde(default)] name: String) -> QueryResult<Self::Error>;
}

pub struct SomeContract {}

#[contract(error=ContractError)]
impl SomeContract {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }
    #[msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(query, resp=QueryResponse)]
    fn contract_query(
        &self,
        _ctx: QueryCtx,
        #[serde(default)] _name: String,
    ) -> QueryResult<ContractError> {
        Ok(QueryResponse {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_interface_query() {
        let _ = msg::InterfaceQueryMsg::Query {
            name: "some_name".to_owned(),
        };
    }

    #[test]
    fn generate_contract_query() {
        let _ = QueryMsg::ContractQuery {
            _name: "some_name".to_owned(),
        };
    }
}

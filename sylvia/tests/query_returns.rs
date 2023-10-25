use cosmwasm_std::{Response, StdError, StdResult};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sylvia::contract;
use sylvia::types::{InstantiateCtx, QueryCtx};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct QueryResponse;

type QueryResult<E> = Result<QueryResponse, E>;

mod msg {
    use cosmwasm_std::StdError;
    use sylvia::{interface, types::QueryCtx};

    use crate::{QueryResponse, QueryResult};

    #[interface(module=msg)]
    pub trait Interface {
        type Error: From<StdError>;

        #[msg(query, resp=QueryResponse)]
        fn query(&self, ctx: QueryCtx, #[serde(default)] name: String) -> QueryResult<Self::Error>;
    }
}

pub struct SomeContract {}

#[contract]
#[error(ContractError)]
impl SomeContract {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
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
        let _ = msg::sv::InterfaceQueryMsg::Query {
            name: "some_name".to_owned(),
        };
    }

    #[test]
    fn generate_contract_query() {
        let _ = sv::QueryMsg::ContractQuery {
            _name: "some_name".to_owned(),
        };
    }
}

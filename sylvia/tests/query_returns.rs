use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};

use sylvia::{contract, interface};
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
    fn query(&self, ctx: (Deps, Env)) -> QueryResult<Self::Error>;
}

pub struct SomeContract {}

#[contract(error=ContractError)]
impl SomeContract {
    #[msg(instantiate)]
    pub fn instantiate(&self, _ctx: (DepsMut, Env, MessageInfo)) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(query, resp=QueryResponse)]
    fn contract_query(&self, _ctx: (Deps, Env)) -> QueryResult<ContractError> {
        Ok(QueryResponse {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_interface_query() {
        let _ = msg::InterfaceQueryMsg::Query {};
    }

    #[test]
    fn generate_contract_query() {
        let _ = QueryMsg::ContractQuery {};
    }
}

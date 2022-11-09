use cosmwasm_std::{Deps, Env, StdError};

use sylvia::interface;
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

    #[msg(query, QueryResponse)]
    fn custom_result_query(&self, ctx: (Deps, Env)) -> QueryResult<Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_query() {
        let _ = msg::InterfaceQueryMsg::CustomResultQuery {};
    }
}

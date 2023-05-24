use cosmwasm_std::{Addr, Decimal, Response, StdError, StdResult};

use sylvia::types::{ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx};
use sylvia::{contract, interface};

#[derive(
    serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema,
)]
pub struct QueryResult;

#[interface(module=interface)]
pub trait Interface {
    type Error: From<StdError>;

    #[msg(exec)]
    fn no_args_execution(&self, ctx: ExecCtx) -> Result<Response, Self::Error>;

    #[msg(exec)]
    fn argumented_execution(
        &self,
        ctx: ExecCtx,
        addr: Addr,
        coef: Decimal,
        #[serde(default)] desc: String,
    ) -> Result<Response, Self::Error>;

    #[msg(query)]
    fn no_args_query(&self, ctx: QueryCtx) -> Result<QueryResult, Self::Error>;

    #[msg(query)]
    fn argumented_query(&self, ctx: QueryCtx, user: Addr) -> Result<QueryResult, Self::Error>;
}

pub struct Contract {}

#[cfg(not(tarpaulin_include))]
// Ignoring coverage of test implementation
#[contract(module=contract)]
impl Contract {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    #[msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(migrate)]
    pub fn migrate(&self, _ctx: MigrateCtx) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(exec)]
    fn no_args_execution(&self, _ctx: ExecCtx) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(exec)]
    fn argumented_execution(
        &self,
        _ctx: ExecCtx,
        _addr: Addr,
        #[serde(default)] _coef: Decimal,
        #[serde(default)] _desc: String,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(query)]
    fn no_args_query(&self, _ctx: QueryCtx) -> StdResult<QueryResult> {
        Ok(QueryResult {})
    }

    #[msg(query)]
    fn argumented_query(&self, _ctx: QueryCtx, _user: Addr) -> StdResult<QueryResult> {
        Ok(QueryResult {})
    }
}

#[test]
fn interface_messages_constructible() {
    let no_args_exec = interface::ExecMsg::NoArgsExecution {};
    let _argumented_exec = interface::ExecMsg::ArgumentedExecution {
        addr: Addr::unchecked("owner"),
        coef: Decimal::percent(10),
        desc: "Some description".to_owned(),
    };
    let no_args_query = interface::QueryMsg::NoArgsQuery {};
    let _argumented_query = interface::QueryMsg::ArgumentedQuery {
        user: Addr::unchecked("owner"),
    };

    // Ensure no extra variants are generated
    match no_args_exec {
        interface::ExecMsg::NoArgsExecution {} => (),
        interface::ExecMsg::ArgumentedExecution { .. } => (),
    }

    match no_args_query {
        interface::QueryMsg::NoArgsQuery {} => (),
        interface::QueryMsg::ArgumentedQuery { .. } => (),
    }
}

#[test]
fn contract_messages_constructible() {
    let no_args_exec = contract::ExecMsg::NoArgsExecution {};
    let _argumented_exec = contract::ExecMsg::ArgumentedExecution {
        _addr: Addr::unchecked("owner"),
        _coef: Decimal::percent(10),
        _desc: "Some description".to_owned(),
    };
    let no_args_query = contract::QueryMsg::NoArgsQuery {};
    let _argumented_query = contract::QueryMsg::ArgumentedQuery {
        _user: Addr::unchecked("owner"),
    };
    let _ = contract::InstantiateMsg {};
    let _ = contract::MigrateMsg {};

    // Ensure no extra variants are generated
    match no_args_exec {
        contract::ExecMsg::NoArgsExecution {} => (),
        contract::ExecMsg::ArgumentedExecution { .. } => (),
    }

    match no_args_query {
        contract::QueryMsg::NoArgsQuery {} => (),
        contract::QueryMsg::ArgumentedQuery { .. } => (),
    }
}

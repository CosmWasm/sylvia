use cosmwasm_std::{Addr, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};

use sylvia::{contract, interface};

#[derive(
    serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema,
)]
pub struct QueryResult;

#[interface(module=interface)]
pub trait Interface {
    type Error: From<StdError>;

    #[msg(exec)]
    fn no_args_execution(&self, ctx: (DepsMut, Env, MessageInfo)) -> Result<Response, Self::Error>;

    #[msg(exec)]
    fn argumented_execution(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        addr: Addr,
        coef: Decimal,
        desc: String,
    ) -> Result<Response, Self::Error>;

    #[msg(query)]
    fn no_args_query(&self, ctx: (Deps, Env)) -> Result<QueryResult, Self::Error>;

    #[msg(query)]
    fn argumented_query(&self, ctx: (Deps, Env), user: Addr) -> Result<QueryResult, Self::Error>;
}

pub struct Contract {}

#[cfg(not(tarpaulin_include))]
// Ignoring coverage of test implementation
#[contract(module=contract, error=StdError)]
impl Contract {
    #[msg(instantiate)]
    pub fn instantiate(&self, _ctx: (DepsMut, Env, MessageInfo)) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(migrate)]
    pub fn migrate(&self, _ctx: (DepsMut, Env)) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(reply)]
    pub fn reply(&self, _ctx: (DepsMut, Env)) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(exec)]
    fn no_args_execution(&self, _ctx: (DepsMut, Env, MessageInfo)) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[allow(unused_variables)]
    #[msg(exec)]
    fn argumented_execution(
        &self,
        _ctx: (DepsMut, Env, MessageInfo),
        addr: Addr,
        coef: Decimal,
        desc: String,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(query)]
    fn no_args_query(&self, _ctx: (Deps, Env)) -> StdResult<QueryResult> {
        Ok(QueryResult {})
    }

    #[allow(unused_variables)]
    #[msg(query)]
    fn argumented_query(&self, _ctx: (Deps, Env), user: Addr) -> StdResult<QueryResult> {
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
        addr: Addr::unchecked("owner"),
        coef: Decimal::percent(10),
        desc: "Some description".to_owned(),
    };
    let no_args_query = contract::QueryMsg::NoArgsQuery {};
    let _argumented_query = contract::QueryMsg::ArgumentedQuery {
        user: Addr::unchecked("owner"),
    };
    let _ = contract::InstantiateMsg {};
    let reply = contract::ReplyMsg::Reply {};
    let migrate = contract::MigrateMsg::Migrate {};

    // Ensure no extra variants are generated
    match no_args_exec {
        contract::ExecMsg::NoArgsExecution {} => (),
        contract::ExecMsg::ArgumentedExecution { .. } => (),
    }

    match no_args_query {
        contract::QueryMsg::NoArgsQuery {} => (),
        contract::QueryMsg::ArgumentedQuery { .. } => (),
    }

    match reply {
        contract::ReplyMsg::Reply {} => (),
    }

    match migrate {
        contract::MigrateMsg::Migrate {} => (),
    }
}

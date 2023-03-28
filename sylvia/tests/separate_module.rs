use cosmwasm_std::{Addr, Decimal, Response, StdError};

use sylvia::{
    interface,
    types::{ExecCtx, QueryCtx},
};

#[derive(
    serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema,
)]
pub struct QueryResult;

#[interface(module=msg)]
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
        desc: String,
    ) -> Result<Response, Self::Error>;

    #[msg(query)]
    fn no_args_query(&self, ctx: QueryCtx) -> Result<QueryResult, Self::Error>;

    #[msg(query)]
    fn argumented_query(
        &self,
        ctx: QueryCtx,
        user: Addr,
    ) -> Result<Option<QueryResult>, Self::Error>;
}

#[test]
fn messages_constructible() {
    let _no_args_exec = msg::ExecMsg::NoArgsExecution {};
    let _argumented_exec = msg::ExecMsg::ArgumentedExecution {
        addr: Addr::unchecked("owner"),
        coef: Decimal::percent(10),
        desc: "Some description".to_owned(),
    };
    let _no_args_query = msg::QueryMsg::NoArgsQuery {};
    let _argumented_query = msg::QueryMsg::ArgumentedQuery {
        user: Addr::unchecked("owner"),
    };
}

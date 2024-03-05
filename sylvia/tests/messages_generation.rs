use cosmwasm_std::{Addr, CustomQuery, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema,
)]
pub struct QueryResult;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
pub struct MyQuery;

impl CustomQuery for MyQuery {}

mod interface {
    use cosmwasm_std::{Addr, Decimal, Response, StdError};
    use sylvia::interface;
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};

    use crate::QueryResult;

    #[interface]
    pub trait Interface {
        type Error: From<StdError>;

        #[sv::msg(exec)]
        fn no_args_execution(&self, ctx: ExecCtx) -> Result<Response, Self::Error>;

        #[sv::msg(exec)]
        fn argumented_execution(
            &self,
            ctx: ExecCtx,
            addr: Addr,
            coef: Decimal,
            #[serde(default)] desc: String,
        ) -> Result<Response, Self::Error>;

        #[sv::msg(query)]
        fn no_args_query(&self, ctx: QueryCtx) -> Result<QueryResult, Self::Error>;

        #[sv::msg(query)]
        fn argumented_query(&self, ctx: QueryCtx, user: Addr) -> Result<QueryResult, Self::Error>;

        #[sv::msg(sudo)]
        fn no_args_sudo(&self, ctx: SudoCtx) -> Result<Response, Self::Error>;

        #[sv::msg(sudo)]
        fn argumented_sudo(&self, ctx: SudoCtx, user: Addr) -> Result<Response, Self::Error>;
    }
}

mod contract {
    use cosmwasm_std::{Addr, Reply, Response, StdResult};
    use sylvia::contract;
    use sylvia::types::{ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx, ReplyCtx, SudoCtx};
    use sylvia_derive::entry_points;

    use crate::{MyQuery, QueryResult};

    pub struct Contract {}

    #[entry_points]
    #[contract]
    #[allow(dead_code)]
    #[sv::custom(query=MyQuery)]
    impl Contract {
        #[allow(clippy::new_without_default)]
        #[allow(dead_code)]
        pub fn new() -> Self {
            Self {}
        }

        #[sv::msg(instantiate)]
        pub fn instantiate(&self, _ctx: InstantiateCtx<MyQuery>) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(migrate)]
        pub fn migrate(&self, _ctx: MigrateCtx<MyQuery>) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(exec)]
        fn no_args_execution(&self, _ctx: ExecCtx<MyQuery>) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(exec)]
        fn argumented_execution(
            &self,
            _ctx: ExecCtx<MyQuery>,
            _addr: cosmwasm_std::Addr,
            #[serde(default)] _coef: cosmwasm_std::Decimal,
            #[serde(default)] _desc: String,
        ) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(query)]
        fn no_args_query(&self, _ctx: QueryCtx<MyQuery>) -> StdResult<QueryResult> {
            Ok(QueryResult {})
        }

        #[sv::msg(query)]
        fn argumented_query(&self, _ctx: QueryCtx<MyQuery>, _user: Addr) -> StdResult<QueryResult> {
            Ok(QueryResult {})
        }

        #[sv::msg(reply)]
        fn my_reply(&self, _ctx: ReplyCtx<MyQuery>, _reply: Reply) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(sudo)]
        fn no_args_sudo(&self, _ctx: SudoCtx<MyQuery>) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(sudo)]
        fn argumented_sudo(&self, _ctx: SudoCtx<MyQuery>, _user: Addr) -> StdResult<Response> {
            Ok(Response::new())
        }
    }
}

#[test]
fn interface_messages_constructible() {
    let no_args_exec = interface::sv::ExecMsg::NoArgsExecution {};
    let _ = interface::sv::ExecMsg::ArgumentedExecution {
        addr: Addr::unchecked("owner"),
        coef: Decimal::percent(10),
        desc: "Some description".to_owned(),
    };
    let no_args_query = interface::sv::QueryMsg::NoArgsQuery {};
    let _ = interface::sv::QueryMsg::ArgumentedQuery {
        user: Addr::unchecked("owner"),
    };
    let no_args_sudo = interface::sv::SudoMsg::NoArgsSudo {};
    let _ = interface::sv::SudoMsg::ArgumentedSudo {
        user: Addr::unchecked("owner"),
    };

    // Ensure no extra variants are generated
    match no_args_exec {
        interface::sv::ExecMsg::NoArgsExecution {} => (),
        interface::sv::ExecMsg::ArgumentedExecution { .. } => (),
    }

    match no_args_query {
        interface::sv::QueryMsg::NoArgsQuery {} => (),
        interface::sv::QueryMsg::ArgumentedQuery { .. } => (),
    }

    match no_args_sudo {
        interface::sv::SudoMsg::NoArgsSudo {} => (),
        interface::sv::SudoMsg::ArgumentedSudo { .. } => (),
    }
}

#[test]
fn contract_messages_constructible() {
    let no_args_exec = contract::sv::ExecMsg::NoArgsExecution {};
    let _argumented_exec = contract::sv::ExecMsg::ArgumentedExecution {
        _addr: Addr::unchecked("owner"),
        _coef: Decimal::percent(10),
        _desc: "Some description".to_owned(),
    };
    let no_args_query = contract::sv::QueryMsg::NoArgsQuery {};
    let _argumented_query = contract::sv::QueryMsg::ArgumentedQuery {
        _user: Addr::unchecked("owner"),
    };
    let no_args_sudo = contract::sv::SudoMsg::NoArgsSudo {};
    let _ = contract::sv::SudoMsg::ArgumentedSudo {
        _user: Addr::unchecked("owner"),
    };
    let _ = contract::sv::InstantiateMsg {};
    let _ = contract::sv::MigrateMsg {};

    // Ensure no extra variants are generated
    match no_args_exec {
        contract::sv::ExecMsg::NoArgsExecution {} => (),
        contract::sv::ExecMsg::ArgumentedExecution { .. } => (),
    }

    match no_args_query {
        contract::sv::QueryMsg::NoArgsQuery {} => (),
        contract::sv::QueryMsg::ArgumentedQuery { .. } => (),
    }

    match no_args_sudo {
        contract::sv::SudoMsg::NoArgsSudo {} => (),
        contract::sv::SudoMsg::ArgumentedSudo { .. } => (),
    }
}

#[test]
fn entry_points_generation() {
    use contract::entry_points;

    let _ = cw_multi_test::ContractWrapper::new(
        entry_points::execute,
        entry_points::instantiate,
        entry_points::query,
    )
    .with_migrate(entry_points::migrate)
    .with_reply(entry_points::reply)
    .with_sudo(entry_points::sudo);
}

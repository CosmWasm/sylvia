use std::fmt::Debug;
use std::str::FromStr;

use contract::sv::{ExecMsg, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg};
use cosmwasm_std::{from_json, Addr, CustomQuery, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema,
)]
pub struct QueryResult;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
pub struct MyQuery;

impl CustomQuery for MyQuery {}

pub mod interface {
    use cosmwasm_std::{Addr, Decimal, Response, StdError};
    use sylvia::interface;
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};
    use thiserror::Error;

    use crate::QueryResult;

    #[interface]
    #[sv::msg_attr(exec, derive(PartialOrd, Error))]
    #[sv::msg_attr(query, derive(PartialOrd, Error))]
    #[sv::msg_attr(sudo, derive(PartialOrd, Error))]
    #[sv::custom(msg=cosmwasm_std::Empty, query=cosmwasm_std::Empty)]
    pub trait Interface {
        type Error: From<StdError>;

        #[sv::msg(exec)]
        #[sv::attr(error("NoArgsExecution"))]
        fn no_args_execution(&self, ctx: ExecCtx) -> Result<Response, Self::Error>;

        #[sv::msg(exec)]
        #[sv::attr(error("ArgumentedExecution"))]
        fn argumented_execution(
            &self,
            ctx: ExecCtx,
            addr: Addr,
            coef: Decimal,
            #[serde(default)] desc: String,
        ) -> Result<Response, Self::Error>;

        #[sv::msg(query)]
        #[sv::attr(error("NoArgsQuery"))]
        fn no_args_query(&self, ctx: QueryCtx) -> Result<QueryResult, Self::Error>;

        #[sv::msg(query)]
        #[sv::attr(error("ArgumentedQuery"))]
        fn argumented_query(&self, ctx: QueryCtx, user: Addr) -> Result<QueryResult, Self::Error>;

        #[sv::msg(sudo)]
        #[sv::attr(error("NoArgsSudo"))]
        fn no_args_sudo(&self, ctx: SudoCtx) -> Result<Response, Self::Error>;

        #[sv::msg(sudo)]
        #[sv::attr(error("ArgumentedSudo"))]
        fn argumented_sudo(&self, ctx: SudoCtx, user: Addr) -> Result<Response, Self::Error>;
    }
}

mod contract {
    use cosmwasm_std::{Addr, Reply, Response, StdResult};
    use sylvia::contract;
    use sylvia::types::{ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx, ReplyCtx, SudoCtx};
    use sylvia_derive::entry_points;
    use thiserror::Error;

    use crate::{MyQuery, QueryResult};

    pub struct Contract {}

    #[entry_points]
    #[contract]
    #[allow(dead_code)]
    #[sv::custom(query=MyQuery)]
    #[sv::msg_attr(exec, derive(PartialOrd, Error))]
    #[sv::msg_attr(query, derive(PartialOrd, Error))]
    #[sv::msg_attr(sudo, derive(PartialOrd, Error))]
    #[sv::msg_attr(instantiate, derive(PartialOrd, Error))]
    #[sv::msg_attr(instantiate, error("Instantiate"))]
    #[sv::msg_attr(migrate, derive(PartialOrd, Error))]
    #[sv::msg_attr(migrate, error("Migrate"))]
    impl Contract {
        #[allow(clippy::new_without_default)]
        #[allow(dead_code)]
        pub fn new() -> Self {
            Self {}
        }

        #[sv::msg(instantiate)]
        pub fn instantiate(
            &self,
            _ctx: InstantiateCtx<MyQuery>,
            #[serde(default)] _desc: String,
        ) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(migrate)]
        pub fn migrate(
            &self,
            _ctx: MigrateCtx<MyQuery>,
            #[serde(default)] _desc: String,
        ) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(exec)]
        #[sv::attr(error("NoArgsExecution"))]
        fn no_args_execution(&self, _ctx: ExecCtx<MyQuery>) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(exec)]
        #[sv::attr(error("ArgumentedExecution"))]
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
        #[sv::attr(error("NoArgsQuery"))]
        fn no_args_query(&self, _ctx: QueryCtx<MyQuery>) -> StdResult<QueryResult> {
            Ok(QueryResult {})
        }

        #[sv::msg(query)]
        #[sv::attr(error("ArgumentedQuery"))]
        fn argumented_query(
            &self,
            _ctx: QueryCtx<MyQuery>,
            _user: Addr,
            #[serde(default)] _desc: String,
        ) -> StdResult<QueryResult> {
            Ok(QueryResult {})
        }

        #[sv::msg(reply)]
        fn my_reply(&self, _ctx: ReplyCtx<MyQuery>, _reply: Reply) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(sudo)]
        #[sv::attr(error("NoArgsSudo"))]
        fn no_args_sudo(&self, _ctx: SudoCtx<MyQuery>) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(sudo)]
        #[sv::attr(error("ArgumentedSudo"))]
        fn argumented_sudo(
            &self,
            _ctx: SudoCtx<MyQuery>,
            _user: Addr,
            #[serde(default)] _desc: String,
        ) -> StdResult<Response> {
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
        _desc: "".to_string(),
    };
    let no_args_sudo = contract::sv::SudoMsg::NoArgsSudo {};
    let _ = contract::sv::SudoMsg::ArgumentedSudo {
        _user: Addr::unchecked("owner"),
        _desc: "".to_string(),
    };
    let _ = contract::sv::InstantiateMsg {
        _desc: "".to_string(),
    };
    let _ = contract::sv::MigrateMsg {
        _desc: "".to_string(),
    };

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

#[test]
fn attributes_forwarding_for_message_structs_and_enums() {
    let exec_msg_contract = contract::sv::ExecMsg::NoArgsExecution {};
    let _partial_ord_implemented = exec_msg_contract < exec_msg_contract;

    let query_msg_contract = contract::sv::QueryMsg::NoArgsQuery {};
    let _partial_ord_implemented = query_msg_contract < query_msg_contract;

    let sudo_msg_contract = contract::sv::SudoMsg::NoArgsSudo {};
    let _partial_ord_implemented = sudo_msg_contract < sudo_msg_contract;

    let instantiate_msg_contract = contract::sv::InstantiateMsg {
        _desc: "".to_string(),
    };
    let _partial_ord_implemented = instantiate_msg_contract < instantiate_msg_contract;

    let migrate_msg_contract = contract::sv::MigrateMsg {
        _desc: "".to_string(),
    };
    let _partial_ord_implemented = migrate_msg_contract < migrate_msg_contract;

    let exec_msg_interface = interface::sv::ExecMsg::NoArgsExecution {};
    let _partial_ord_implemented = exec_msg_interface < exec_msg_interface;

    let query_msg_interface = interface::sv::QueryMsg::NoArgsQuery {};
    let _partial_ord_implemented = query_msg_interface < query_msg_interface;

    let sudo_msg_interface = interface::sv::SudoMsg::NoArgsSudo {};
    let _partial_ord_implemented = sudo_msg_interface < sudo_msg_interface;
}

#[test]
fn attributes_forwarding_for_message_variants() {
    // By adding #[sv::msg_attr(exec, derive(Error))] and #[sv::attr(error("NoArgsExecution"))]
    // for execution message and its variants, it should be possible to call `to_string` method
    // on each exec variant. Without this derive, the method is not implemented.
    // Analogous for the other message enums and structs.

    assert_eq!(
        contract::sv::ExecMsg::NoArgsExecution {}.to_string(),
        "NoArgsExecution"
    );
    assert_eq!(
        contract::sv::ExecMsg::ArgumentedExecution {
            _addr: Addr::unchecked("input"),
            _coef: Decimal::from_str("0").unwrap(),
            _desc: "".to_owned(),
        }
        .to_string(),
        "ArgumentedExecution"
    );
    assert_eq!(
        contract::sv::QueryMsg::NoArgsQuery {}.to_string(),
        "NoArgsQuery"
    );
    assert_eq!(
        contract::sv::QueryMsg::ArgumentedQuery {
            _user: Addr::unchecked("input"),
            _desc: "".to_string(),
        }
        .to_string(),
        "ArgumentedQuery"
    );
    assert_eq!(
        contract::sv::SudoMsg::NoArgsSudo {}.to_string(),
        "NoArgsSudo"
    );
    assert_eq!(
        contract::sv::SudoMsg::ArgumentedSudo {
            _user: Addr::unchecked("input"),
            _desc: "".to_string(),
        }
        .to_string(),
        "ArgumentedSudo"
    );
    assert_eq!(
        contract::sv::InstantiateMsg {
            _desc: "".to_string()
        }
        .to_string(),
        "Instantiate"
    );
    assert_eq!(
        contract::sv::MigrateMsg {
            _desc: "".to_string(),
        }
        .to_string(),
        "Migrate"
    );

    assert_eq!(
        interface::sv::ExecMsg::NoArgsExecution {}.to_string(),
        "NoArgsExecution"
    );
    assert_eq!(
        interface::sv::ExecMsg::ArgumentedExecution {
            addr: Addr::unchecked("input"),
            coef: Decimal::from_str("0").unwrap(),
            desc: "".to_owned(),
        }
        .to_string(),
        "ArgumentedExecution"
    );
    assert_eq!(
        interface::sv::QueryMsg::NoArgsQuery {}.to_string(),
        "NoArgsQuery"
    );
    assert_eq!(
        interface::sv::QueryMsg::ArgumentedQuery {
            user: Addr::unchecked("input"),
        }
        .to_string(),
        "ArgumentedQuery"
    );
    assert_eq!(
        interface::sv::SudoMsg::NoArgsSudo {}.to_string(),
        "NoArgsSudo"
    );
    assert_eq!(
        interface::sv::SudoMsg::ArgumentedSudo {
            user: Addr::unchecked("input"),
        }
        .to_string(),
        "ArgumentedSudo"
    );
}

/// When deserializing some fields should be created using [Default::default] even though they are
/// missing
#[test]
fn forward_attribute_to_message_field() {
    let json = br#"{
            "instantiate": {}
        }"#;

    assert_eq!(
        InstantiateMsg::new(Default::default()),
        from_json(json).unwrap()
    );

    let json = br#"{
            "argumented_query": {"_user" : "addr"}
        }"#;

    assert_eq!(
        QueryMsg::argumented_query(Addr::unchecked("addr"), Default::default()),
        from_json(json).unwrap()
    );

    let json = br#"{
            "argumented_sudo": {"_user" : "addr"}
        }"#;

    assert_eq!(
        SudoMsg::argumented_sudo(Addr::unchecked("addr"), Default::default()),
        from_json(json).unwrap()
    );

    let json = br#"{
            "argumented_execution": {"_addr" : "addr"}
        }"#;

    assert_eq!(
        ExecMsg::argumented_execution(
            Addr::unchecked("addr"),
            Default::default(),
            Default::default()
        ),
        from_json(json).unwrap()
    );

    let json = br#"{
            "migrate": {}
        }"#;

    assert_eq!(
        MigrateMsg::new(Default::default()),
        from_json(json).unwrap()
    );
}

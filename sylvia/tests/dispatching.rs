use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_json, Addr, Decimal, Response};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::contract::Contract;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct EmptyQueryResponse {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct QueryResponse {
    coef: Decimal,
    desc: String,
}

mod interface {
    use cosmwasm_std::{Addr, Decimal, Response, StdError};
    use sylvia::interface;
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};

    use crate::{EmptyQueryResponse, QueryResponse};

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
            desc: String,
        ) -> Result<Response, Self::Error>;

        #[sv::msg(query)]
        fn no_args_query(&self, ctx: QueryCtx) -> Result<EmptyQueryResponse, Self::Error>;

        #[sv::msg(query)]
        fn argumented_query(&self, ctx: QueryCtx, user: Addr)
            -> Result<QueryResponse, Self::Error>;

        #[sv::msg(sudo)]
        fn no_args_sudo(&self, ctx: SudoCtx) -> Result<Response, Self::Error>;

        #[sv::msg(sudo)]
        fn argumented_sudo(&self, ctx: SudoCtx, user: Addr) -> Result<Response, Self::Error>;
    }
}

mod impl_interface {
    use cosmwasm_std::{Addr, Decimal, Response, StdError};
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};

    use crate::{EmptyQueryResponse, QueryResponse};

    #[sylvia::contract(module = crate::contract)]
    #[sv::messages(crate::interface)]
    impl crate::interface::Interface for crate::contract::Contract {
        type Error = StdError;

        #[sv::msg(exec)]
        fn no_args_execution(&self, ctx: ExecCtx) -> Result<Response, StdError> {
            self.execs
                .update(ctx.deps.storage, |count| -> Result<u64, StdError> {
                    Ok(count + 1)
                })?;
            Ok(Response::new())
        }

        #[sv::msg(exec)]
        fn argumented_execution(
            &self,
            ctx: ExecCtx,
            addr: Addr,
            coef: Decimal,
            desc: String,
        ) -> Result<Response, Self::Error> {
            self.execs
                .update(ctx.deps.storage, |count| -> Result<u64, StdError> {
                    Ok(count + 1)
                })?;
            self.data
                .save(ctx.deps.storage, addr, &QueryResponse { coef, desc })?;

            Ok(Response::new())
        }

        #[sv::msg(query)]
        fn no_args_query(&self, _: QueryCtx) -> Result<EmptyQueryResponse, StdError> {
            *self.queries.borrow_mut() += 1;
            Ok(dbg!(EmptyQueryResponse {}))
        }

        #[sv::msg(query)]
        fn argumented_query(
            &self,
            ctx: QueryCtx,
            user: Addr,
        ) -> Result<QueryResponse, Self::Error> {
            *self.queries.borrow_mut() += 1;
            Ok(self.data.load(ctx.deps.storage, user).unwrap())
        }

        #[sv::msg(sudo)]
        fn no_args_sudo(&self, ctx: SudoCtx) -> Result<Response, Self::Error> {
            self.sudos
                .update(ctx.deps.storage, |count| -> Result<u64, StdError> {
                    Ok(count + 1)
                })?;
            Ok(Response::new())
        }

        #[sv::msg(sudo)]
        fn argumented_sudo(&self, ctx: SudoCtx, user: Addr) -> Result<Response, Self::Error> {
            self.sudos
                .update(ctx.deps.storage, |count| -> Result<u64, StdError> {
                    Ok(count + 1)
                })?;
            let resp = Response::new().add_attribute("user", user);
            Ok(resp)
        }
    }
}

mod contract {
    use std::cell::RefCell;

    use cosmwasm_std::{Addr, Response, StdError, StdResult};
    use cw_storage_plus::{Item, Map};
    use sylvia::types::{InstantiateCtx, SudoCtx};
    use sylvia_derive::{contract, entry_points};

    use crate::QueryResponse;

    pub struct Contract {
        pub(crate) execs: Item<'static, u64>,
        pub(crate) queries: RefCell<u64>,
        pub(crate) sudos: Item<'static, u64>,

        pub(crate) data: Map<'static, Addr, QueryResponse>,
    }

    #[entry_points]
    #[allow(dead_code)]
    #[contract]
    #[sv::messages(crate::interface)]
    impl Contract {
        pub fn new() -> Self {
            Self {
                execs: Item::new("execs"),
                queries: RefCell::default(),
                sudos: Item::new("sudos"),
                data: Map::new("data"),
            }
        }

        #[sv::msg(instantiate)]
        fn instanciate(&self, ctx: InstantiateCtx) -> StdResult<Response> {
            self.execs.save(ctx.deps.storage, &0)?;
            self.sudos.save(ctx.deps.storage, &0)?;
            Ok(Response::new())
        }

        #[sv::msg(sudo)]
        fn contract_sudo(&self, ctx: SudoCtx) -> StdResult<Response> {
            self.sudos
                .update(ctx.deps.storage, |count| -> Result<u64, StdError> {
                    Ok(count + 1)
                })?;
            Ok(Response::new())
        }
    }
}

#[test]
fn dispatch() {
    let contract = Contract::new();

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("owner", &[]);

    // Instantiate the contract
    let resp = contract::sv::InstantiateMsg {}
        .dispatch(&contract, (deps.as_mut(), env.clone(), info.clone()))
        .unwrap();
    assert_eq!(resp, Response::new());

    // Execs
    let resp = interface::sv::ExecMsg::NoArgsExecution {}
        .dispatch(&contract, (deps.as_mut(), env.clone(), info.clone()))
        .unwrap();
    assert_eq!(resp, Response::new());

    let resp = interface::sv::ExecMsg::ArgumentedExecution {
        addr: Addr::unchecked("addr1"),
        coef: Decimal::percent(30),
        desc: "True".to_owned(),
    }
    .dispatch(&contract, (deps.as_mut(), env.clone(), info.clone()))
    .unwrap();
    assert_eq!(resp, Response::new());

    let resp = interface::sv::ExecMsg::ArgumentedExecution {
        addr: Addr::unchecked("addr2"),
        coef: Decimal::percent(70),
        desc: "False".to_owned(),
    }
    .dispatch(&contract, (deps.as_mut(), env.clone(), info))
    .unwrap();
    assert_eq!(resp, Response::new());

    // Queries
    let resp = interface::sv::QueryMsg::NoArgsQuery {}
        .dispatch(&contract, (deps.as_ref(), env.clone()))
        .unwrap();
    let _resp: EmptyQueryResponse = from_json(resp).unwrap();

    let resp = interface::sv::QueryMsg::ArgumentedQuery {
        user: Addr::unchecked("addr2"),
    }
    .dispatch(&contract, (deps.as_ref(), env.clone()))
    .unwrap();
    let resp: QueryResponse = from_json(resp).unwrap();
    assert_eq!(
        resp,
        QueryResponse {
            coef: Decimal::percent(70),
            desc: "False".to_owned()
        }
    );

    // Sudos
    let resp = interface::sv::SudoMsg::NoArgsSudo {}
        .dispatch(&contract, (deps.as_mut(), env.clone()))
        .unwrap();
    assert_eq!(resp, Response::new());

    let resp = interface::sv::SudoMsg::ArgumentedSudo {
        user: Addr::unchecked("addr2"),
    }
    .dispatch(&contract, (deps.as_mut(), env.clone()))
    .unwrap();
    assert_eq!(
        resp,
        Response::new().add_attribute("user", "addr2".to_owned())
    );

    let resp = contract::sv::SudoMsg::ContractSudo {}
        .dispatch(&contract, (deps.as_mut(), env))
        .unwrap();
    assert_eq!(resp, Response::new());

    assert_eq!(contract.execs.load(&deps.storage).unwrap(), 3);
    assert_eq!(*contract.queries.borrow(), 2);
    assert_eq!(contract.sudos.load(&deps.storage).unwrap(), 3);
}

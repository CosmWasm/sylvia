use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Addr, Decimal, Response};
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
    use sylvia::types::{ExecCtx, QueryCtx};

    use crate::{EmptyQueryResponse, QueryResponse};

    #[interface]
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
        fn no_args_query(&self, ctx: QueryCtx) -> Result<EmptyQueryResponse, Self::Error>;

        #[msg(query)]
        fn argumented_query(&self, ctx: QueryCtx, user: Addr)
            -> Result<QueryResponse, Self::Error>;
    }
}

mod impl_interface {
    use cosmwasm_std::{Addr, Decimal, Response, StdError};
    use sylvia::types::{ExecCtx, QueryCtx};

    use crate::{EmptyQueryResponse, QueryResponse};

    #[sylvia::contract(module = crate::contract)]
    #[messages(crate::interface as Interface)]
    impl crate::interface::Interface for crate::contract::Contract {
        type Error = StdError;

        #[msg(exec)]
        fn no_args_execution(&self, _: ExecCtx) -> Result<Response, StdError> {
            *self.execs.borrow_mut() += 1;
            Ok(Response::new())
        }

        #[msg(exec)]
        fn argumented_execution(
            &self,
            _: ExecCtx,
            addr: Addr,
            coef: Decimal,
            desc: String,
        ) -> Result<Response, Self::Error> {
            *self.execs.borrow_mut() += 1;

            self.data
                .borrow_mut()
                .insert(addr, QueryResponse { coef, desc });
            Ok(Response::new())
        }

        #[msg(query)]
        fn no_args_query(&self, _: QueryCtx) -> Result<EmptyQueryResponse, StdError> {
            *self.queries.borrow_mut() += 1;
            Ok(dbg!(EmptyQueryResponse {}))
        }

        #[msg(query)]
        fn argumented_query(&self, _: QueryCtx, user: Addr) -> Result<QueryResponse, Self::Error> {
            *self.queries.borrow_mut() += 1;
            Ok(self.data.borrow().get(&user).unwrap().clone())
        }
    }
}

mod contract {
    use std::{cell::RefCell, collections::HashMap};

    use cosmwasm_std::{Addr, Response, StdResult};
    use sylvia::types::ExecCtx;
    use sylvia_derive::contract;

    use crate::QueryResponse;

    #[derive(Default)]
    pub struct Contract {
        pub(crate) execs: RefCell<u64>,
        pub(crate) queries: RefCell<u64>,

        pub(crate) data: RefCell<HashMap<Addr, QueryResponse>>,
    }

    #[allow(dead_code)]
    #[cfg(not(tarpaulin_include))]
    #[contract]
    #[messages(crate::interface as Interface)]
    impl Contract {
        fn new() -> Self {
            Self::default()
        }

        #[msg(instantiate)]
        fn instanciate(&self, _: ExecCtx) -> StdResult<Response> {
            Ok(Response::new())
        }
    }
}

#[test]
fn dispatch() {
    let contract = Contract::default();

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("owner", &[]);

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

    let resp = interface::sv::QueryMsg::NoArgsQuery {}
        .dispatch(&contract, (deps.as_ref(), env.clone()))
        .unwrap();
    let _resp: EmptyQueryResponse = from_binary(&resp).unwrap();

    let resp = interface::sv::QueryMsg::ArgumentedQuery {
        user: Addr::unchecked("addr2"),
    }
    .dispatch(&contract, (deps.as_ref(), env))
    .unwrap();
    let resp: QueryResponse = from_binary(&resp).unwrap();
    assert_eq!(
        resp,
        QueryResponse {
            coef: Decimal::percent(70),
            desc: "False".to_owned()
        }
    );

    assert_eq!(*contract.execs.borrow(), 3);
    assert_eq!(*contract.queries.borrow(), 2);
}

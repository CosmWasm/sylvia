#![allow(unused_imports)]

use sylvia::cw_std::{Response, StdResult};
use sylvia::types::{CustomMsg, CustomQuery, InstantiateCtx};
use sylvia::{contract, entry_points};

pub mod no_generics {
    use super::*;

    pub struct Contract<E, Q> {
        _phantom: std::marker::PhantomData<(E, Q)>,
    }

    #[entry_points]
    #[contract]
    #[sv::custom(msg = E, query = Q)]
    impl<E, Q> Contract<E, Q>
    where
        E: CustomMsg + 'static,
        Q: CustomQuery + 'static,
    {
        pub fn new() -> Self {
            Self {
                _phantom: std::marker::PhantomData,
            }
        }

        #[sv::msg(instantiate)]
        fn instantiate(&self, _ctx: InstantiateCtx<Q>) -> StdResult<Response<E>> {
            Ok(Response::new())
        }
    }
}

pub mod missing_generics {
    use super::*;

    pub struct Contract<E, Q> {
        _phantom: std::marker::PhantomData<(E, Q)>,
    }

    #[entry_points(generics<Empty>)]
    #[contract]
    #[sv::custom(msg = E, query = Q)]
    impl<E, Q> Contract<E, Q>
    where
        E: CustomMsg + 'static,
        Q: CustomQuery + 'static,
    {
        pub fn new() -> Self {
            Self {
                _phantom: std::marker::PhantomData,
            }
        }

        #[sv::msg(instantiate)]
        fn instantiate(&self, _ctx: InstantiateCtx<Q>) -> StdResult<Response<E>> {
            Ok(Response::new())
        }
    }
}

fn main() {}

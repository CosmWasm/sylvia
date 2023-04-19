use sylvia::contract;

mod missing_instantiation {
    use super::*;

    pub struct Contract {}

    #[contract]
    impl Contract {
        pub fn new() -> Self {
            Self {}
        }
    }
}

mod missing_new {
    use super::*;

    pub struct Contract {}

    #[contract]
    impl Contract {
        #[msg(instantiate)]
        pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
            Ok(Response::default())
        }
    }
}

fn main() {}

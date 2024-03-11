use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Response, StdResult};
use sylvia::contract;
use sylvia::types::InstantiateCtx;

pub struct SomeContract;

pub mod some_interface {
    use cosmwasm_std::StdError;
    use sylvia::interface;

    #[interface]
    #[sv::custom(msg=cosmwasm_std::Empty, query=cosmwasm_std::Empty)]
    pub trait SomeInterface {
        type Error: From<StdError>;
    }

    impl SomeInterface for super::SomeContract {
        type Error = StdError;
    }
}

#[contract]
#[sv::messages(some_interface)]
impl SomeContract {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::new())
    }
}

// Making sure `Remote` can be stored in `#[cw_serde]` types
#[cw_serde]
#[allow(dead_code)]
struct CustomStorage<Contract> {
    remote: sylvia::types::Remote<'static, Contract>,
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;

    #[test]
    fn remote_generation() {
        // interface
        let _ = sylvia::types::Remote::<()>::new(Addr::unchecked("some_interface"));
        let addr = Addr::unchecked("some_interface");
        let _ = sylvia::types::Remote::<()>::borrowed(&addr);

        // contract
        let addr = Addr::unchecked("some_contract");
        let borrowed_remote = sylvia::types::Remote::<()>::borrowed(&addr);
        assert_eq!(&Addr::unchecked("some_contract"), borrowed_remote.as_ref());
    }
}

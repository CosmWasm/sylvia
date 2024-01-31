use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Response, StdResult};
use sylvia::contract;
use sylvia::types::InstantiateCtx;

pub struct SomeContract;

pub mod some_interface {
    use cosmwasm_std::StdError;
    use sylvia::interface;

    #[interface]
    pub trait SomeInterface {
        type Error: From<StdError>;
    }

    impl SomeInterface for super::SomeContract {
        type Error = StdError;
    }
}

#[contract]
#[messages(some_interface)]
impl SomeContract {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    #[msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::new())
    }
}

// Making sure `Remote` can be stored in `#[cw_serde]` types
#[cw_serde]
#[allow(dead_code)]
struct CustomStorage {
    remote: crate::sv::Remote<'static>,
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;

    use crate::some_interface;

    #[test]
    fn remote_generation() {
        // interface
        let _ = some_interface::sv::Remote::new(Addr::unchecked("some_interface"));
        let addr = Addr::unchecked("some_interface");
        let _ = some_interface::sv::Remote::borrowed(&addr);

        // contract
        let new_remote = crate::sv::Remote::new(Addr::unchecked("some_contract"));
        let addr = Addr::unchecked("some_contract");
        let borrowed_remote = crate::sv::Remote::borrowed(&addr);
        assert_eq!(&Addr::unchecked("some_contract"), borrowed_remote.as_ref());

        let _ = some_interface::sv::Remote::from(&borrowed_remote);
        let _ = some_interface::sv::Remote::from(&new_remote);
    }
}

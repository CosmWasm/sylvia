use cosmwasm_std::{Binary, Response, StdError, Uint128};
use sylvia::types::ExecCtx;
use sylvia::{interface, schemars};

#[interface]
#[sv::custom(msg=cosmwasm_std::Empty, query=cosmwasm_std::Empty)]
pub trait Receiver {
    type Error: From<StdError>;

    #[sv::msg(exec)]
    fn receive(
        &self,
        ctx: ExecCtx,
        sender: String,
        amount: Uint128,
        msg: Binary,
    ) -> Result<Response, Self::Error>;
}

pub mod impl_receiver {
    use crate::multitest::receiver_contract::ReceiverContract;
    use cosmwasm_std::{Response, StdError};
    use sylvia::types::ExecCtx;

    impl super::Receiver for ReceiverContract {
        type Error = StdError;

        fn receive(
            &self,
            _ctx: ExecCtx,
            _sender: String,
            _amount: cosmwasm_std::Uint128,
            _msg: cosmwasm_std::Binary,
        ) -> Result<Response, Self::Error> {
            Ok(Response::default())
        }
    }
}

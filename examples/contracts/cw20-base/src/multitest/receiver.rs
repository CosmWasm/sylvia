use sylvia::cw_std::{Binary, Response, StdError, Uint128};
use sylvia::interface;
use sylvia::types::ExecCtx;

#[interface]
#[sv::custom(msg=sylvia::cw_std::Empty, query=sylvia::cw_std::Empty)]
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
    use sylvia::cw_std::{Response, StdError};
    use sylvia::types::ExecCtx;

    impl super::Receiver for ReceiverContract {
        type Error = StdError;

        fn receive(
            &self,
            _ctx: ExecCtx,
            _sender: String,
            _amount: sylvia::cw_std::Uint128,
            _msg: sylvia::cw_std::Binary,
        ) -> Result<Response, Self::Error> {
            Ok(Response::default())
        }
    }
}

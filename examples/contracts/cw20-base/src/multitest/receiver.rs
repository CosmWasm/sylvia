use cosmwasm_std::{Binary, Response, StdError, Uint128};
use sylvia::types::ExecCtx;
use sylvia::{interface, schemars};

#[interface]
pub trait Receiver {
    type Error: From<StdError>;

    #[msg(exec)]
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
    use sylvia::contract;
    use sylvia::types::ExecCtx;

    #[contract(module=crate::multitest::receiver_contract)]
    #[messages(crate::multitest::receiver as Receiver)]
    impl super::Receiver for ReceiverContract {
        type Error = StdError;

        #[msg(exec)]
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

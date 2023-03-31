use cosmwasm_std::{Binary, Response, StdError, Uint128};
use sylvia::types::ExecCtx;
use sylvia::{contract, interface, schemars};

use super::receiver_contract::ReceiverContract;

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

#[contract]
impl Receiver for ReceiverContract {
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

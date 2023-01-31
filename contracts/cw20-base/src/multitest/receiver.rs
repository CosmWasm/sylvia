use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdError, Uint128};
use sylvia::{contract, interface, schemars};

use super::receiver_contract::ReceiverContract;

#[interface]
pub trait Receiver {
    type Error: From<StdError>;

    #[msg(exec)]
    fn receive(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        sender: String,
        amount: Uint128,
        msg: Binary,
    ) -> Result<Response, Self::Error>;
}

#[contract]
impl Receiver for ReceiverContract {
    type Error = StdError;

    fn receive(
        &self,
        _ctx: (DepsMut, Env, MessageInfo),
        _sender: String,
        _amount: cosmwasm_std::Uint128,
        _msg: cosmwasm_std::Binary,
    ) -> Result<Response, Self::Error> {
        Ok(Response::default())
    }
}

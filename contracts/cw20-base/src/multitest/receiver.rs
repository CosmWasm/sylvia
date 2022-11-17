use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdError, Uint128};
use sylvia::interface;

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

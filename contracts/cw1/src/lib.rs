#![allow(dead_code)]
use cosmwasm_std::{CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError};

use sylvia_derive::interface;

#[interface(module=msg)]
pub trait Cw1<Msg>
where
    Msg: std::fmt::Debug + PartialEq + Clone + schemars::JsonSchema,
{
    type Error: From<StdError>;

    #[msg(exec)]
    fn execute(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        msgs: Vec<CosmosMsg<Msg>>,
    ) -> Result<Response, Self::Error>;

    #[msg(query)]
    fn query(&self, ctx: (Deps, Env), addr: String) -> Result<Response, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}

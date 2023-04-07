use cosmwasm_std::{CosmosMsg, Response};

use sylvia::types::ExecCtx;
use sylvia_derive::interface;

#[interface(module=msg)]
pub trait Cw1<Msg>
where
    Msg: std::fmt::Debug + PartialEq + Clone + schemars::JsonSchema,
{
    type Error;

    #[msg(exec)]
    fn execute(&self, ctx: ExecCtx, msgs: Vec<CosmosMsg<Msg>>) -> Result<Response, Self::Error>;
}

fn main() {}

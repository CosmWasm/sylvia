use cosmwasm_std::{CosmosMsg, CustomMsg, Response};

use sylvia::types::ExecCtx;
use sylvia_derive::interface;

#[interface(module=msg)]
pub trait Cw1<Msg>
where
    Msg: std::fmt::Debug + PartialEq + Clone + schemars::JsonSchema,
{
    type Error;
    type ExecC: CustomMsg;

    #[msg(exec)]
    fn execute(
        &self,
        ctx: ExecCtx,
        msgs: Vec<CosmosMsg<Msg>>,
    ) -> Result<Response<Self::ExecC>, Self::Error>;
}

fn main() {}

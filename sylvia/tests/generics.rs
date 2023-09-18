use cosmwasm_schema::cw_serde;

pub mod cw1 {
    use cosmwasm_std::{CosmosMsg, CustomMsg, Response, StdError};

    use serde::Deserialize;
    use sylvia::types::{ExecCtx, QueryCtx};
    use sylvia_derive::interface;

    #[interface(module=msg)]
    pub trait Cw1<Msg, Param>
    where
        for<'msg_de> Msg: CustomMsg + Deserialize<'msg_de>,
        Param: sylvia::types::CustomMsg,
    {
        type Error: From<StdError>;

        #[msg(exec)]
        fn execute(&self, ctx: ExecCtx, msgs: Vec<CosmosMsg<Msg>>)
            -> Result<Response, Self::Error>;

        #[msg(query)]
        fn query(&self, ctx: QueryCtx, param: Param) -> Result<String, Self::Error>;
    }
}

#[cw_serde]
pub struct ExternalMsg;
impl cosmwasm_std::CustomMsg for ExternalMsg {}
impl sylvia::types::CustomMsg for ExternalMsg {}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{CosmosMsg, Empty};

    use crate::ExternalMsg;

    #[test]
    fn construct_messages() {
        let _ = crate::cw1::QueryMsg::query(ExternalMsg {});
        let _ = crate::cw1::ExecMsg::execute(vec![CosmosMsg::Custom(ExternalMsg {})]);
        let _ = crate::cw1::ExecMsg::execute(vec![CosmosMsg::Custom(Empty {})]);
    }
}

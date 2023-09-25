use cosmwasm_schema::cw_serde;

pub mod cw1 {
    use cosmwasm_std::{CosmosMsg, CustomMsg, CustomQuery, Response, StdError};

    use serde::{de::DeserializeOwned, Deserialize};
    use sylvia::types::{ExecCtx, QueryCtx};
    use sylvia_derive::interface;

    #[interface(module=msg)]
    #[sv::custom(msg=Msg)]
    pub trait Cw1<Msg, Param, QueryRet>
    where
        for<'msg_de> Msg: CustomMsg + Deserialize<'msg_de>,
        Param: sylvia::types::CustomMsg,
        for<'msg_de> QueryRet: CustomQuery + DeserializeOwned,
    {
        type Error: From<StdError>;

        #[msg(exec)]
        fn execute(
            &self,
            ctx: ExecCtx,
            msgs: Vec<CosmosMsg<Msg>>,
        ) -> Result<Response<Msg>, Self::Error>;

        #[msg(query)]
        fn some_query(&self, ctx: QueryCtx, param: Param) -> Result<QueryRet, Self::Error>;
    }
}

#[cw_serde]
pub struct ExternalMsg;
impl cosmwasm_std::CustomMsg for ExternalMsg {}

#[cw_serde]
pub struct ExternalQuery;
impl cosmwasm_std::CustomQuery for ExternalQuery {}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::mock_dependencies, Addr, CosmosMsg, Empty, QuerierWrapper};

    use crate::{cw1::Querier, ExternalMsg, ExternalQuery};

    use crate::cw1::InterfaceTypes;
    use sylvia::types::InterfaceMessages;
    #[test]
    fn construct_messages() {
        let contract = Addr::unchecked("contract");

        let _ = crate::cw1::QueryMsg::<_, Empty>::some_query(ExternalMsg {});
        let _ = crate::cw1::ExecMsg::execute(vec![CosmosMsg::Custom(ExternalMsg {})]);
        let _ = crate::cw1::ExecMsg::execute(vec![CosmosMsg::Custom(Empty {})]);

        // Generic Querier
        let deps = mock_dependencies();
        let querier: QuerierWrapper<ExternalQuery> = QuerierWrapper::new(&deps.querier);

        let cw1_querier = crate::cw1::BoundQuerier::borrowed(&contract, &querier);
        let _: Result<ExternalQuery, _> = Querier::some_query(&cw1_querier, ExternalMsg {});
        let _: Result<ExternalQuery, _> = cw1_querier.some_query(ExternalMsg {});

        // Construct messages with Interface extension
        let _ =
            <InterfaceTypes<ExternalMsg, _, ExternalQuery> as InterfaceMessages>::Query::some_query(
                ExternalMsg {},
            );
        let _=
            <InterfaceTypes<_, ExternalMsg, cosmwasm_std::Empty> as InterfaceMessages>::Exec::execute(vec![
                CosmosMsg::Custom(ExternalMsg {}),
            ]);
    }
}

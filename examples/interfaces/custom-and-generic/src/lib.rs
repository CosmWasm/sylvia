use cosmwasm_std::{CosmosMsg, CustomMsg, Response, StdError};

use serde::de::DeserializeOwned;
use serde::Deserialize;
use sylvia::types::{ExecCtx, QueryCtx};
use sylvia::{interface, schemars};

#[interface]
#[sv::custom(msg=RetType)]
pub trait CustomAndGeneric<ExecParam, QueryParam, RetType>
where
    for<'msg_de> ExecParam: CustomMsg + Deserialize<'msg_de>,
    QueryParam: sylvia::types::CustomMsg,
    RetType: CustomMsg + DeserializeOwned,
{
    type Error: From<StdError>;

    #[msg(exec)]
    fn custom_generic_execute(
        &self,
        ctx: ExecCtx,
        msgs: Vec<CosmosMsg<ExecParam>>,
    ) -> Result<Response<RetType>, Self::Error>;

    #[msg(query)]
    fn custom_generic_query(
        &self,
        ctx: QueryCtx,
        param: QueryParam,
    ) -> Result<RetType, Self::Error>;
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{Addr, CosmosMsg, Empty, QuerierWrapper};
    use sylvia::types::{InterfaceApi, SvCustomMsg};

    use crate::sv::Querier;

    #[test]
    fn construct_messages() {
        let contract = Addr::unchecked("contract");

        // Direct message construction
        let _ = super::sv::QueryMsg::<_, Empty>::custom_generic_query(SvCustomMsg {});
        let _ = super::sv::ExecMsg::custom_generic_execute(vec![CosmosMsg::Custom(SvCustomMsg {})]);
        let _ = super::sv::ExecMsg::custom_generic_execute(vec![CosmosMsg::Custom(SvCustomMsg {})]);

        // Querier
        let deps = mock_dependencies();
        let querier_wrapper: QuerierWrapper = QuerierWrapper::new(&deps.querier);

        let querier = super::sv::BoundQuerier::borrowed(&contract, &querier_wrapper);
        let _: Result<SvCustomMsg, _> =
            super::sv::Querier::custom_generic_query(&querier, SvCustomMsg {});
        let _: Result<SvCustomMsg, _> = querier.custom_generic_query(SvCustomMsg {});

        // Construct messages with Interface extension
        let _ =
            <super::sv::Api<SvCustomMsg, _, SvCustomMsg> as InterfaceApi>::Query::custom_generic_query(
                SvCustomMsg {},
            );
        let _=
            <super::sv::Api<_, SvCustomMsg, cosmwasm_std::Empty> as InterfaceApi>::Exec::custom_generic_execute(
            vec![ CosmosMsg::Custom(SvCustomMsg{}),
            ]);
    }
}

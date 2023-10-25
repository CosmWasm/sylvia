use cosmwasm_std::{CosmosMsg, CustomMsg, Response, StdError};

use serde::{de::DeserializeOwned, Deserialize};
use sylvia::types::{ExecCtx, QueryCtx};
use sylvia::{interface, schemars};

#[interface]
pub trait Generic<ExecParam, QueryParam, RetType>
where
    for<'msg_de> ExecParam: CustomMsg + Deserialize<'msg_de>,
    QueryParam: sylvia::types::CustomMsg,
    RetType: CustomMsg + DeserializeOwned,
{
    type Error: From<StdError>;

    #[msg(exec)]
    fn generic_exec(
        &self,
        ctx: ExecCtx,
        msgs: Vec<CosmosMsg<ExecParam>>,
    ) -> Result<Response, Self::Error>;

    #[msg(query)]
    fn generic_query(&self, ctx: QueryCtx, param: QueryParam) -> Result<RetType, Self::Error>;
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::mock_dependencies, Addr, CosmosMsg, Empty, QuerierWrapper};
    use sylvia::types::{InterfaceMessages, SvCustomMsg};

    use crate::sv::Querier;

    #[test]
    fn construct_messages() {
        let contract = Addr::unchecked("contract");

        // Direct message construction
        let _ = super::sv::QueryMsg::<_, Empty>::generic_query(SvCustomMsg {});
        let _ = super::sv::ExecMsg::generic_exec(vec![CosmosMsg::Custom(SvCustomMsg {})]);
        let _ = super::sv::ExecMsg::generic_exec(vec![CosmosMsg::Custom(SvCustomMsg {})]);

        // Querier
        let deps = mock_dependencies();
        let querier_wrapper: QuerierWrapper = QuerierWrapper::new(&deps.querier);

        let querier = super::sv::BoundQuerier::borrowed(&contract, &querier_wrapper);
        let _: Result<SvCustomMsg, _> = super::sv::Querier::generic_query(&querier, SvCustomMsg {});
        let _: Result<SvCustomMsg, _> = querier.generic_query(SvCustomMsg {});

        // Construct messages with Interface extension
        let _ =
            <super::sv::InterfaceTypes<SvCustomMsg, _, SvCustomMsg> as InterfaceMessages>::Query::generic_query(
                SvCustomMsg {},
            );
        let _=
            <super::sv::InterfaceTypes<_, SvCustomMsg, cosmwasm_std::Empty> as InterfaceMessages>::Exec::generic_exec(vec![
                CosmosMsg::Custom(SvCustomMsg{}),
            ]);
    }
}

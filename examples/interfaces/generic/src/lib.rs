use cosmwasm_std::{CosmosMsg, CustomMsg, Response, StdError};

use serde::{de::DeserializeOwned, Deserialize};
use sylvia::types::{ExecCtx, QueryCtx};
use sylvia::{interface, schemars};

#[interface]
pub trait Generic<Exec1T, Exec2T, Exec3T, QueryT, RetT>
where
    for<'msg_de> Exec1T: CustomMsg + Deserialize<'msg_de>,
    Exec2T: sylvia::types::CustomMsg,
    Exec3T: sylvia::types::CustomMsg,
    QueryT: sylvia::types::CustomMsg,
    RetT: CustomMsg + DeserializeOwned,
{
    type Error: From<StdError>;

    #[msg(exec)]
    fn generic_exec_one(
        &self,
        ctx: ExecCtx,
        msgs1: Vec<CosmosMsg<Exec1T>>,
        msgs2: Vec<CosmosMsg<Exec2T>>,
    ) -> Result<Response, Self::Error>;

    #[msg(exec)]
    fn generic_exec_two(
        &self,
        ctx: ExecCtx,
        msgs1: Vec<CosmosMsg<Exec2T>>,
        msgs2: Vec<CosmosMsg<Exec3T>>,
    ) -> Result<Response, Self::Error>;

    #[msg(query)]
    fn generic_query(&self, ctx: QueryCtx, param: QueryT) -> Result<RetT, Self::Error>;
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::mock_dependencies, Addr, CosmosMsg, Empty, QuerierWrapper};
    use sylvia::types::{InterfaceApi, SvCustomMsg};

    use crate::sv::Querier;

    #[test]
    fn construct_messages() {
        let contract = Addr::unchecked("contract");

        // Direct message construction
        let _ = super::sv::QueryMsg::<_, Empty>::generic_query(SvCustomMsg {});
        let _ = super::sv::ExecMsg::<_, _, SvCustomMsg>::generic_exec_one(
            vec![CosmosMsg::Custom(SvCustomMsg {})],
            vec![CosmosMsg::Custom(SvCustomMsg {})],
        );
        let _ = super::sv::ExecMsg::<SvCustomMsg, _, _>::generic_exec_two(
            vec![CosmosMsg::Custom(SvCustomMsg {})],
            vec![CosmosMsg::Custom(SvCustomMsg {})],
        );

        // Querier
        let deps = mock_dependencies();
        let querier_wrapper: QuerierWrapper = QuerierWrapper::new(&deps.querier);

        let querier = super::sv::BoundQuerier::borrowed(&contract, &querier_wrapper);
        let _: Result<SvCustomMsg, _> = super::sv::Querier::generic_query(&querier, SvCustomMsg {});
        let _: Result<SvCustomMsg, _> = querier.generic_query(SvCustomMsg {});

        // Construct messages with Interface extension
        let _ = <super::sv::Api<SvCustomMsg, SvCustomMsg, SvCustomMsg, _, SvCustomMsg> as InterfaceApi>::Query::generic_query(
            SvCustomMsg {},
        );
        let _=
            <super::sv::Api<_, _, SvCustomMsg, SvCustomMsg, cosmwasm_std::Empty> as InterfaceApi>::Exec::generic_exec_one(vec![
                CosmosMsg::Custom(SvCustomMsg{}),
            ],vec![
                CosmosMsg::Custom(SvCustomMsg{}),
            ]
        );
    }
}

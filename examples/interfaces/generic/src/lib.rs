use cosmwasm_std::{CosmosMsg, Response, StdError};

use serde::Deserialize;
use sylvia::types::{CustomMsg, ExecCtx, QueryCtx};
use sylvia::{interface, schemars};

#[interface]
pub trait Generic<Exec1T, Exec2T, Exec3T, Query1T, Query2T, Query3T, RetT>
where
    for<'msg_de> Exec1T: CustomMsg + Deserialize<'msg_de>,
    Exec2T: CustomMsg,
    Exec3T: CustomMsg,
    Query1T: CustomMsg,
    Query2T: CustomMsg,
    Query3T: CustomMsg,
    RetT: CustomMsg,
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
    fn generic_query_one(
        &self,
        ctx: QueryCtx,
        param1: Query1T,
        param2: Query2T,
    ) -> Result<RetT, Self::Error>;

    #[msg(query)]
    fn generic_query_two(
        &self,
        ctx: QueryCtx,
        param1: Query2T,
        param2: Query3T,
    ) -> Result<RetT, Self::Error>;
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::mock_dependencies, Addr, CosmosMsg, Empty, QuerierWrapper};
    use sylvia::types::{InterfaceApi, SvCustomMsg};

    #[test]
    fn construct_messages() {
        let contract = Addr::unchecked("contract");

        // Direct message construction
        let _ = super::sv::QueryMsg::<_, _, Empty, SvCustomMsg>::generic_query_one(
            SvCustomMsg {},
            SvCustomMsg {},
        );
        let _ = super::sv::QueryMsg::<SvCustomMsg, _, Empty, _>::generic_query_two(
            SvCustomMsg {},
            SvCustomMsg {},
        );
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
        let _: Result<SvCustomMsg, _> =
            super::sv::Querier::<_, _, _, SvCustomMsg>::generic_query_one(
                &querier,
                SvCustomMsg {},
                SvCustomMsg {},
            );
        let _: Result<SvCustomMsg, _> =
            super::sv::Querier::<SvCustomMsg, _, _, _>::generic_query_two(
                &querier,
                SvCustomMsg {},
                SvCustomMsg {},
            );
        // let _: Result<SvCustomMsg, _> = querier.generic_query_one(SvCustomMsg {}, SvCustomMsg {});
        // let _: Result<SvCustomMsg, _> = querier.generic_query_two(SvCustomMsg {}, SvCustomMsg {});

        // Construct messages with Interface extension
        let _ = <super::sv::Api<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            _,
            _,
            SvCustomMsg,
            SvCustomMsg,
        > as InterfaceApi>::Query::generic_query_one(SvCustomMsg {}, SvCustomMsg {});
        let _ = <super::sv::Api<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            _,
            _,
            SvCustomMsg,
        > as InterfaceApi>::Query::generic_query_two(SvCustomMsg {}, SvCustomMsg {});
        let _ = <super::sv::Api<
            _,
            _,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            cosmwasm_std::Empty,
        > as InterfaceApi>::Exec::generic_exec_one(
            vec![CosmosMsg::Custom(SvCustomMsg {})],
            vec![CosmosMsg::Custom(SvCustomMsg {})],
        );
        let _ = <super::sv::Api<
            SvCustomMsg,
            _,
            _,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            cosmwasm_std::Empty,
        > as InterfaceApi>::Exec::generic_exec_two(
            vec![CosmosMsg::Custom(SvCustomMsg {})],
            vec![CosmosMsg::Custom(SvCustomMsg {})],
        );
    }
}

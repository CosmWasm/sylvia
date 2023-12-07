use cosmwasm_std::{CosmosMsg, CustomMsg, Response, StdError};

use serde::de::DeserializeOwned;
use serde::Deserialize;
use sylvia::types::{ExecCtx, QueryCtx};
use sylvia::{interface, schemars};

#[interface]
#[sv::custom(msg=CustomMsgT, query=CustomQueryT)]
pub trait CustomAndGeneric<
    Exec1T,
    Exec2T,
    Exec3T,
    Query1T,
    Query2T,
    Query3T,
    RetT,
    CustomMsgT,
    CustomQueryT,
> where
    for<'msg_de> Exec1T: CustomMsg + Deserialize<'msg_de>,
    Exec2T: sylvia::types::CustomMsg,
    Exec3T: sylvia::types::CustomMsg,
    Query1T: sylvia::types::CustomMsg,
    Query2T: sylvia::types::CustomMsg,
    Query3T: sylvia::types::CustomMsg,
    RetT: CustomMsg + DeserializeOwned,
    CustomMsgT: CustomMsg + DeserializeOwned,
    CustomQueryT: sylvia::types::CustomQuery + 'static,
{
    type Error: From<StdError>;

    #[msg(exec)]
    fn custom_generic_execute_one(
        &self,
        ctx: ExecCtx<CustomQueryT>,
        msgs1: Vec<CosmosMsg<Exec1T>>,
        msgs2: Vec<CosmosMsg<Exec2T>>,
    ) -> Result<Response<CustomMsgT>, Self::Error>;

    #[msg(exec)]
    fn custom_generic_execute_two(
        &self,
        ctx: ExecCtx<CustomQueryT>,
        msgs1: Vec<CosmosMsg<Exec2T>>,
        msgs2: Vec<CosmosMsg<Exec3T>>,
    ) -> Result<Response<CustomMsgT>, Self::Error>;

    #[msg(query)]
    fn custom_generic_query_one(
        &self,
        ctx: QueryCtx<CustomQueryT>,
        param1: Query1T,
        param2: Query2T,
    ) -> Result<RetT, Self::Error>;

    #[msg(query)]
    fn custom_generic_query_two(
        &self,
        ctx: QueryCtx<CustomQueryT>,
        param1: Query2T,
        param2: Query3T,
    ) -> Result<RetT, Self::Error>;
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{Addr, CosmosMsg, Empty, QuerierWrapper};
    use sylvia::types::{InterfaceApi, SvCustomMsg, SvCustomQuery};

    #[test]
    fn construct_messages() {
        let contract = Addr::unchecked("contract");

        // Direct message construction
        let _ = super::sv::QueryMsg::<_, _, Empty, SvCustomMsg>::custom_generic_query_one(
            SvCustomMsg {},
            SvCustomMsg {},
        );
        let _ = super::sv::QueryMsg::<SvCustomMsg, _, Empty, _>::custom_generic_query_two(
            SvCustomMsg {},
            SvCustomMsg {},
        );

        let _ = super::sv::ExecMsg::<_, _, SvCustomMsg>::custom_generic_execute_one(
            vec![CosmosMsg::Custom(SvCustomMsg {})],
            vec![CosmosMsg::Custom(SvCustomMsg {})],
        );
        let _ = super::sv::ExecMsg::<SvCustomMsg, _, _>::custom_generic_execute_two(
            vec![CosmosMsg::Custom(SvCustomMsg {})],
            vec![CosmosMsg::Custom(SvCustomMsg {})],
        );

        // Querier
        let deps = mock_dependencies();
        let querier_wrapper: QuerierWrapper = QuerierWrapper::new(&deps.querier);

        let querier = super::sv::BoundQuerier::borrowed(&contract, &querier_wrapper);

        let _: Result<SvCustomMsg, _> =
            super::sv::Querier::<_, _, _, SvCustomMsg>::custom_generic_query_one(
                &querier,
                SvCustomMsg {},
                SvCustomMsg {},
            );
        // let _: Result<SvCustomMsg, _> =
        //     querier.custom_generic_query_one(SvCustomMsg {}, SvCustomMsg {});
        let _: Result<SvCustomMsg, _> =
            super::sv::Querier::<SvCustomMsg, _, _, _>::custom_generic_query_two(
                &querier,
                SvCustomMsg {},
                SvCustomMsg {},
            );
        // let _: Result<SvCustomMsg, _> =
        //     querier.custom_generic_query_two(SvCustomMsg {}, SvCustomMsg {});

        // Construct messages with Interface extension
        let _ = <super::sv::Api<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomQuery,
        > as InterfaceApi>::Query::custom_generic_query_one(
            SvCustomMsg {}, SvCustomMsg {}
        );

        let _ = <super::sv::Api<
            _,
            _,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            cosmwasm_std::Empty,
            SvCustomQuery,
        > as InterfaceApi>::Exec::custom_generic_execute_one(
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
            SvCustomMsg,
            cosmwasm_std::Empty,
            SvCustomQuery,
        > as InterfaceApi>::Exec::custom_generic_execute_two(
            vec![CosmosMsg::Custom(SvCustomMsg {})],
            vec![CosmosMsg::Custom(SvCustomMsg {})],
        );
    }
}

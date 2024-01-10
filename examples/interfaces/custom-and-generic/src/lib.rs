use cosmwasm_std::{CosmosMsg, Response, StdError};

use sylvia::types::{CustomMsg, CustomQuery, ExecCtx, QueryCtx};
use sylvia::{interface, schemars};

#[interface]
#[sv::custom(msg=CustomMsgT, query=CustomQueryT)]
pub trait CustomAndGeneric {
    type Error: From<StdError>;
    type Exec1T: CustomMsg;
    type Exec2T: CustomMsg;
    type Exec3T: CustomMsg;
    type Query1T: CustomMsg;
    type Query2T: CustomMsg;
    type Query3T: CustomMsg;
    type CustomMsgT: CustomMsg;
    type CustomQueryT: CustomQuery + 'static;
    type RetT: CustomMsg;

    #[msg(exec)]
    fn custom_generic_execute_one(
        &self,
        ctx: ExecCtx<Self::CustomQueryT>,
        msgs1: Vec<CosmosMsg<Self::Exec1T>>,
        msgs2: Vec<CosmosMsg<Self::Exec2T>>,
    ) -> Result<Response<Self::CustomMsgT>, Self::Error>;

    #[msg(exec)]
    fn custom_generic_execute_two(
        &self,
        ctx: ExecCtx<Self::CustomQueryT>,
        msgs1: Vec<CosmosMsg<Self::Exec2T>>,
        msgs2: Vec<CosmosMsg<Self::Exec3T>>,
    ) -> Result<Response<Self::CustomMsgT>, Self::Error>;

    #[msg(query)]
    fn custom_generic_query_one(
        &self,
        ctx: QueryCtx<Self::CustomQueryT>,
        param1: Self::Query1T,
        param2: Self::Query2T,
    ) -> Result<Self::RetT, Self::Error>;

    #[msg(query)]
    fn custom_generic_query_two(
        &self,
        ctx: QueryCtx<Self::CustomQueryT>,
        param1: Self::Query2T,
        param2: Self::Query3T,
    ) -> Result<Self::RetT, Self::Error>;
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{Addr, CosmosMsg, Empty, QuerierWrapper};
    use sylvia::types::{InterfaceApi, SvCustomMsg, SvCustomQuery};

    use crate::sv::Querier;

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

        let querier = super::sv::BoundQuerier::<
            _,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomQuery,
            SvCustomMsg,
        >::borrowed(&contract, &querier_wrapper);

        let _: Result<SvCustomMsg, _> =
            super::sv::Querier::custom_generic_query_one(&querier, SvCustomMsg {}, SvCustomMsg {});
        let _: Result<SvCustomMsg, _> =
            querier.custom_generic_query_one(SvCustomMsg {}, SvCustomMsg {});
        let _: Result<SvCustomMsg, _> =
            super::sv::Querier::custom_generic_query_two(&querier, SvCustomMsg {}, SvCustomMsg {});
        let _: Result<SvCustomMsg, _> =
            querier.custom_generic_query_two(SvCustomMsg {}, SvCustomMsg {});

        // Construct messages with Interface extension
        let _ = <super::sv::Api<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomQuery,
            SvCustomMsg,
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
            SvCustomQuery,
            cosmwasm_std::Empty,
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
            SvCustomQuery,
            cosmwasm_std::Empty,
        > as InterfaceApi>::Exec::custom_generic_execute_two(
            vec![CosmosMsg::Custom(SvCustomMsg {})],
            vec![CosmosMsg::Custom(SvCustomMsg {})],
        );
    }
}

use sylvia::cw_std::{CosmosMsg, Response, StdError};

use sylvia::interface;
use sylvia::types::{CustomMsg, CustomQuery, ExecCtx, QueryCtx, SudoCtx};

#[interface]
pub trait CustomAndGeneric {
    type Error: From<StdError>;
    type Exec1T: CustomMsg;
    type Exec2T: CustomMsg;
    type Exec3T: CustomMsg;
    type Query1T: CustomMsg;
    type Query2T: CustomMsg;
    type Query3T: CustomMsg;
    type Sudo1T: CustomMsg;
    type Sudo2T: CustomMsg;
    type Sudo3T: CustomMsg;
    type ExecC: CustomMsg;
    type QueryC: CustomQuery + 'static;
    type RetT: CustomMsg;

    #[sv::msg(exec)]
    fn custom_generic_execute_one(
        &self,
        ctx: ExecCtx<Self::QueryC>,
        msgs1: Vec<CosmosMsg<Self::Exec1T>>,
        msgs2: Vec<CosmosMsg<Self::Exec2T>>,
    ) -> Result<Response<Self::ExecC>, Self::Error>;

    #[sv::msg(exec)]
    fn custom_generic_execute_two(
        &self,
        ctx: ExecCtx<Self::QueryC>,
        msgs1: Vec<CosmosMsg<Self::Exec2T>>,
        msgs2: Vec<CosmosMsg<Self::Exec3T>>,
    ) -> Result<Response<Self::ExecC>, Self::Error>;

    #[sv::msg(query)]
    fn custom_generic_query_one(
        &self,
        ctx: QueryCtx<Self::QueryC>,
        param1: Self::Query1T,
        param2: Self::Query2T,
    ) -> Result<Self::RetT, Self::Error>;

    #[sv::msg(query)]
    fn custom_generic_query_two(
        &self,
        ctx: QueryCtx<Self::QueryC>,
        param1: Self::Query2T,
        param2: Self::Query3T,
    ) -> Result<Self::RetT, Self::Error>;

    #[sv::msg(sudo)]
    fn custom_generic_sudo_one(
        &self,
        ctx: SudoCtx<Self::QueryC>,
        msg1: CosmosMsg<Self::Sudo1T>,
        msg2: CosmosMsg<Self::Sudo2T>,
    ) -> Result<Response<Self::ExecC>, Self::Error>;

    #[sv::msg(sudo)]
    fn custom_generic_sudo_two(
        &self,
        ctx: SudoCtx<Self::QueryC>,
        msg1: CosmosMsg<Self::Sudo2T>,
        msg2: CosmosMsg<Self::Sudo3T>,
    ) -> Result<Response<Self::ExecC>, Self::Error>;
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use sylvia::cw_std::testing::mock_dependencies;
    use sylvia::cw_std::{Addr, CosmosMsg, Empty, QuerierWrapper};
    use sylvia::types::InterfaceApi;

    use crate::sv::Querier;

    #[sylvia::cw_schema::cw_serde(crate = "sylvia::cw_schema")]
    pub struct SvCustomMsg;
    impl sylvia::cw_std::CustomMsg for SvCustomMsg {}
    #[sylvia::cw_schema::cw_serde(crate = "sylvia::cw_schema")]
    pub struct SvCustomQuery;
    impl sylvia::cw_std::CustomQuery for SvCustomQuery {}

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

        let querier = sylvia::types::BoundQuerier::<
            _,
            dyn super::CustomAndGeneric<
                RetT = SvCustomMsg,
                Exec1T = SvCustomMsg,
                Exec2T = SvCustomMsg,
                Exec3T = SvCustomMsg,
                Query1T = SvCustomMsg,
                Query2T = SvCustomMsg,
                Query3T = SvCustomMsg,
                Sudo1T = SvCustomMsg,
                Sudo2T = SvCustomMsg,
                Sudo3T = SvCustomMsg,
                Error = (),
                ExecC = SvCustomMsg,
                QueryC = SvCustomQuery,
            >,
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
        let _ = <dyn super::CustomAndGeneric<
            Error = (),
            Exec1T = SvCustomMsg,
            Exec2T = SvCustomMsg,
            Exec3T = SvCustomMsg,
            Query1T = SvCustomMsg,
            Query2T = SvCustomMsg,
            Query3T = SvCustomMsg,
            Sudo1T = SvCustomMsg,
            Sudo2T = SvCustomMsg,
            Sudo3T = SvCustomMsg,
            ExecC = SvCustomMsg,
            QueryC = SvCustomQuery,
            RetT = SvCustomMsg,
        > as super::sv::InterfaceMessagesApi>::Query::custom_generic_query_one(
            SvCustomMsg {},
            SvCustomMsg {},
        );

        let _ = <dyn super::CustomAndGeneric<
            Error = (),
            Exec1T = SvCustomMsg,
            Exec2T = SvCustomMsg,
            Exec3T = SvCustomMsg,
            Query1T = SvCustomMsg,
            Query2T = SvCustomMsg,
            Query3T = SvCustomMsg,
            Sudo1T = SvCustomMsg,
            Sudo2T = SvCustomMsg,
            Sudo3T = SvCustomMsg,
            ExecC = SvCustomMsg,
            QueryC = SvCustomQuery,
            RetT = SvCustomMsg,
        > as super::sv::InterfaceMessagesApi>::Exec::custom_generic_execute_one(
            vec![CosmosMsg::Custom(SvCustomMsg {})],
            vec![CosmosMsg::Custom(SvCustomMsg {})],
        );

        let _ = <dyn super::CustomAndGeneric<
            Error = (),
            Exec1T = SvCustomMsg,
            Exec2T = SvCustomMsg,
            Exec3T = SvCustomMsg,
            Query1T = SvCustomMsg,
            Query2T = SvCustomMsg,
            Query3T = SvCustomMsg,
            Sudo1T = SvCustomMsg,
            Sudo2T = SvCustomMsg,
            Sudo3T = SvCustomMsg,
            ExecC = SvCustomMsg,
            QueryC = SvCustomQuery,
            RetT = SvCustomMsg,
        > as super::sv::InterfaceMessagesApi>::Exec::custom_generic_execute_two(
            vec![CosmosMsg::Custom(SvCustomMsg {})],
            vec![CosmosMsg::Custom(SvCustomMsg {})],
        );

        let _ = <dyn super::CustomAndGeneric<
            Error = (),
            Exec1T = SvCustomMsg,
            Exec2T = SvCustomMsg,
            Exec3T = SvCustomMsg,
            Query1T = SvCustomMsg,
            Query2T = SvCustomMsg,
            Query3T = SvCustomMsg,
            Sudo1T = SvCustomMsg,
            Sudo2T = SvCustomMsg,
            Sudo3T = SvCustomMsg,
            ExecC = SvCustomMsg,
            QueryC = SvCustomQuery,
            RetT = SvCustomMsg,
        > as super::sv::InterfaceMessagesApi>::Sudo::custom_generic_sudo_one(
            CosmosMsg::Custom(SvCustomMsg {}),
            CosmosMsg::Custom(SvCustomMsg {}),
        );

        let _ = <dyn super::CustomAndGeneric<
            Error = (),
            Exec1T = SvCustomMsg,
            Exec2T = SvCustomMsg,
            Exec3T = SvCustomMsg,
            Query1T = SvCustomMsg,
            Query2T = SvCustomMsg,
            Query3T = SvCustomMsg,
            Sudo1T = SvCustomMsg,
            Sudo2T = SvCustomMsg,
            Sudo3T = SvCustomMsg,
            ExecC = SvCustomMsg,
            QueryC = SvCustomQuery,
            RetT = SvCustomMsg,
        > as super::sv::InterfaceMessagesApi>::Sudo::custom_generic_sudo_one(
            CosmosMsg::Custom(SvCustomMsg {}),
            CosmosMsg::Custom(SvCustomMsg {}),
        );

        let _ = <dyn super::CustomAndGeneric<
            Error = (),
            Exec1T = SvCustomMsg,
            Exec2T = SvCustomMsg,
            Exec3T = SvCustomMsg,
            Query1T = SvCustomMsg,
            Query2T = SvCustomMsg,
            Query3T = SvCustomMsg,
            Sudo1T = SvCustomMsg,
            Sudo2T = SvCustomMsg,
            Sudo3T = SvCustomMsg,
            ExecC = SvCustomMsg,
            QueryC = SvCustomQuery,
            RetT = SvCustomMsg,
        > as super::sv::InterfaceMessagesApi>::Querier::borrowed(
            &contract, &querier_wrapper
        );

        // Construct messages with InterfaceApi
        let _ = <super::sv::Api<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
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
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomQuery,
            SvCustomMsg,
        > as InterfaceApi>::Query::custom_generic_query_two(
            SvCustomMsg {}, SvCustomMsg {}
        );

        let _ = <super::sv::Api<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomQuery,
            SvCustomMsg,
        > as InterfaceApi>::Exec::custom_generic_execute_one(
            vec![CosmosMsg::Custom(SvCustomMsg {})],
            vec![CosmosMsg::Custom(SvCustomMsg {})],
        );

        let _ = <super::sv::Api<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomQuery,
            SvCustomMsg,
        > as InterfaceApi>::Exec::custom_generic_execute_two(
            vec![CosmosMsg::Custom(SvCustomMsg {})],
            vec![CosmosMsg::Custom(SvCustomMsg {})],
        );

        let _ = <super::sv::Api<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomQuery,
            SvCustomMsg,
        > as InterfaceApi>::Sudo::custom_generic_sudo_one(
            CosmosMsg::Custom(SvCustomMsg {}),
            CosmosMsg::Custom(SvCustomMsg {}),
        );

        let _ = <super::sv::Api<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomQuery,
            SvCustomMsg,
        > as InterfaceApi>::Sudo::custom_generic_sudo_two(
            CosmosMsg::Custom(SvCustomMsg {}),
            CosmosMsg::Custom(SvCustomMsg {}),
        );
    }
}

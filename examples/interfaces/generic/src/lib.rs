use cosmwasm_std::{CosmosMsg, Response, StdError};

use sylvia::types::{CustomMsg, ExecCtx, QueryCtx, SudoCtx};
use sylvia::{interface, schemars};

#[interface]
#[sv::custom(msg=cosmwasm_std::Empty, query=cosmwasm_std::Empty)]
pub trait Generic {
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
    type RetT: CustomMsg;

    #[sv::msg(exec)]
    fn generic_exec_one(
        &self,
        ctx: ExecCtx,
        msgs1: Vec<CosmosMsg<Self::Exec1T>>,
        msgs2: Vec<CosmosMsg<Self::Exec2T>>,
    ) -> Result<Response, Self::Error>;

    #[sv::msg(exec)]
    fn generic_exec_two(
        &self,
        ctx: ExecCtx,
        msgs1: Vec<CosmosMsg<Self::Exec2T>>,
        msgs2: Vec<CosmosMsg<Self::Exec3T>>,
    ) -> Result<Response, Self::Error>;

    #[sv::msg(query)]
    fn generic_query_one(
        &self,
        ctx: QueryCtx,
        param1: Self::Query1T,
        param2: Self::Query2T,
    ) -> Result<Self::RetT, Self::Error>;

    #[sv::msg(query)]
    fn generic_query_two(
        &self,
        ctx: QueryCtx,
        param1: Self::Query2T,
        param2: Self::Query3T,
    ) -> Result<Self::RetT, Self::Error>;

    #[sv::msg(sudo)]
    fn generic_sudo_one(
        &self,
        ctx: SudoCtx,
        msg1: CosmosMsg<Self::Sudo1T>,
        msg2: CosmosMsg<Self::Sudo2T>,
    ) -> Result<Response, Self::Error>;

    #[sv::msg(sudo)]
    fn generic_sudo_two(
        &self,
        ctx: SudoCtx,
        msg1: CosmosMsg<Self::Sudo2T>,
        msg2: CosmosMsg<Self::Sudo3T>,
    ) -> Result<Response, Self::Error>;
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

        let querier = sylvia::types::BoundQuerier::<
            Empty,
            &dyn super::Generic<
                Exec1T = SvCustomMsg,
                Exec2T = SvCustomMsg,
                Exec3T = SvCustomMsg,
                Query1T = SvCustomMsg,
                Query2T = SvCustomMsg,
                Query3T = SvCustomMsg,
                Sudo1T = SvCustomMsg,
                Sudo2T = SvCustomMsg,
                Sudo3T = SvCustomMsg,
                RetT = SvCustomMsg,
                Error = (),
            >,
        >::borrowed(&contract, &querier_wrapper);
        let _: Result<SvCustomMsg, _> =
            super::sv::Querier::generic_query_one(&querier, SvCustomMsg {}, SvCustomMsg {});
        let _: Result<SvCustomMsg, _> =
            super::sv::Querier::generic_query_two(&querier, SvCustomMsg {}, SvCustomMsg {});
        let _: Result<SvCustomMsg, _> = querier.generic_query_one(SvCustomMsg {}, SvCustomMsg {});
        let _: Result<SvCustomMsg, _> = querier.generic_query_two(SvCustomMsg {}, SvCustomMsg {});

        // Construct messages with Interface extension
        let _ = <super::sv::Api<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            _,
            _,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
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
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
        > as InterfaceApi>::Query::generic_query_two(SvCustomMsg {}, SvCustomMsg {});
        let _ = <super::sv::Api<
            _,
            _,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
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
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
        > as InterfaceApi>::Exec::generic_exec_two(
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
            _,
            _,
            SvCustomMsg,
            cosmwasm_std::Empty,
        > as InterfaceApi>::Sudo::generic_sudo_one(
            CosmosMsg::Custom(SvCustomMsg {}),
            CosmosMsg::Custom(SvCustomMsg {}),
        );
        let _ = <super::sv::Api<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            cosmwasm_std::Empty,
            SvCustomMsg,
            SvCustomMsg,
            _,
            _,
            SvCustomMsg,
        > as InterfaceApi>::Sudo::generic_sudo_two(
            CosmosMsg::Custom(SvCustomMsg {}),
            CosmosMsg::Custom(SvCustomMsg {}),
        );
    }
}

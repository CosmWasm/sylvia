use cosmwasm_std::{CosmosMsg, Response, StdError, StdResult};
use generic::Generic;
use serde::Deserialize;
use sylvia::contract;
use sylvia::types::{CustomMsg, ExecCtx, QueryCtx, SvCustomMsg};

type QueryAlias1 = SvCustomMsg;
type QueryAlias2 = SvCustomMsg;
type QueryAlias3 = SvCustomMsg;

#[contract(module = crate::contract)]
#[messages(generic as Generic)]
#[sv::custom(msg=SvCustomMsg, query=SvCustomQuery)]
impl<InstantiateT, Exec1T, Exec2T, Exec3T, Query1T, Query2T, Query3T, MigrateT, FieldT>
    Generic<
        SvCustomMsg,
        SvCustomMsg,
        SvCustomMsg,
        QueryAlias1,
        QueryAlias2,
        QueryAlias3,
        sylvia::types::SvCustomMsg,
    >
    for crate::contract::GenericContract<
        InstantiateT,
        Exec1T,
        Exec2T,
        Exec3T,
        Query1T,
        Query2T,
        Query3T,
        MigrateT,
        FieldT,
    >
where
    for<'msg_de> InstantiateT: cosmwasm_std::CustomMsg + Deserialize<'msg_de> + 'msg_de,
    Exec1T: CustomMsg + 'static,
    Exec2T: CustomMsg + 'static,
    Exec3T: CustomMsg + 'static,
    Query1T: CustomMsg + 'static,
    Query2T: CustomMsg + 'static,
    Query3T: CustomMsg + 'static,
    MigrateT: CustomMsg + 'static,
    FieldT: 'static,
{
    type Error = StdError;

    #[msg(exec)]
    fn generic_exec_one(
        &self,
        _ctx: ExecCtx,
        _msgs1: Vec<CosmosMsg<SvCustomMsg>>,
        _msgs2: Vec<CosmosMsg<SvCustomMsg>>,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(exec)]
    fn generic_exec_two(
        &self,
        _ctx: ExecCtx,
        _msgs1: Vec<CosmosMsg<SvCustomMsg>>,
        _msgs2: Vec<CosmosMsg<SvCustomMsg>>,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    // Sylvia will fail if single type is used to match against two different generics
    // It's because we have to map unique generics used as they can be used multiple times.
    // If for some reason like here one type would be used in place of two generics either full
    // path or some alias has to be used.
    //
    // Sylvia will fail to recognize generic used if their path is different.
    // F.e. if we this query would return `SvCustomMsg` and we would pass
    // `sylvia::types::SvCustomMsg` to the `Generic` trait paths would not match.
    #[msg(query)]
    fn generic_query_one(
        &self,
        _ctx: QueryCtx,
        _msg1: QueryAlias1,
        _msg2: QueryAlias2,
    ) -> StdResult<sylvia::types::SvCustomMsg> {
        Ok(SvCustomMsg {})
    }

    #[msg(query)]
    fn generic_query_two(
        &self,
        _ctx: QueryCtx,
        _msg1: QueryAlias2,
        _msg2: QueryAlias3,
    ) -> StdResult<sylvia::types::SvCustomMsg> {
        Ok(SvCustomMsg {})
    }
}

#[cfg(test)]
mod tests {
    use super::sv::test_utils::Generic;
    use crate::contract::sv::multitest_utils::CodeId;
    use cosmwasm_std::CosmosMsg;
    use sylvia::multitest::App;
    use sylvia::types::{SvCustomMsg, SvCustomQuery};

    #[test]
    fn proxy_methods() {
        let app = App::<cw_multi_test::BasicApp<SvCustomMsg, SvCustomQuery>>::custom(|_, _, _| {});
        let code_id: CodeId<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            sylvia::types::SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            String,
            _,
        > = CodeId::store_code(&app);

        let owner = "owner";

        let contract = code_id
            .instantiate(SvCustomMsg {})
            .with_label("GenericContract")
            .with_admin(owner)
            .call(owner)
            .unwrap();

        contract
            .generic_exec_one(
                vec![CosmosMsg::Custom(SvCustomMsg {})],
                vec![CosmosMsg::Custom(SvCustomMsg {})],
            )
            .call(owner)
            .unwrap();
        contract
            .generic_exec_two(
                vec![CosmosMsg::Custom(SvCustomMsg {})],
                vec![CosmosMsg::Custom(SvCustomMsg {})],
            )
            .call(owner)
            .unwrap();
        contract
            .generic_query_one(SvCustomMsg, SvCustomMsg)
            .unwrap();
        contract
            .generic_query_two(SvCustomMsg, SvCustomMsg)
            .unwrap();
    }
}

use cosmwasm_std::{CosmosMsg, Response, StdError, StdResult};
use custom_and_generic::CustomAndGeneric;
use sylvia::contract;
use sylvia::types::{ExecCtx, QueryCtx, SvCustomMsg, SvCustomQuery};

type QueryAlias1 = SvCustomMsg;
type QueryAlias2 = SvCustomMsg;
type QueryAlias3 = SvCustomMsg;

#[contract(module = crate::contract)]
#[messages(custom_and_generic as CustomAndGeneric)]
#[sv::custom(msg=SvCustomMsg, query=SvCustomQuery)]
impl<InstantiateT, Exec1T, Exec2T, Exec3T, Query1T, Query2T, Query3T, MigrateT, FieldT>
    CustomAndGeneric<
        SvCustomMsg,
        SvCustomMsg,
        SvCustomMsg,
        SvCustomMsg,
        QueryAlias1,
        QueryAlias2,
        QueryAlias3,
        sylvia::types::SvCustomMsg,
        sylvia::types::SvCustomQuery,
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
{
    type Error = StdError;

    #[msg(exec)]
    fn custom_generic_execute_one(
        &self,
        _ctx: ExecCtx<SvCustomQuery>,
        _msgs1: Vec<CosmosMsg<sylvia::types::SvCustomMsg>>,
        _msgs2: Vec<CosmosMsg<sylvia::types::SvCustomMsg>>,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[msg(exec)]
    fn custom_generic_execute_two(
        &self,
        _ctx: ExecCtx<SvCustomQuery>,
        _msgs1: Vec<CosmosMsg<sylvia::types::SvCustomMsg>>,
        _msgs2: Vec<CosmosMsg<sylvia::types::SvCustomMsg>>,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[msg(query)]
    fn custom_generic_query_one(
        &self,
        _ctx: QueryCtx<SvCustomQuery>,
        _msg1: QueryAlias1,
        _msg2: QueryAlias2,
    ) -> StdResult<SvCustomMsg> {
        Ok(SvCustomMsg {})
    }

    #[msg(query)]
    fn custom_generic_query_two(
        &self,
        _ctx: QueryCtx<SvCustomQuery>,
        _msg1: QueryAlias2,
        _msg2: QueryAlias3,
    ) -> StdResult<SvCustomMsg> {
        Ok(SvCustomMsg {})
    }
}

#[cfg(test)]
mod tests {
    use super::sv::test_utils::CustomAndGeneric;
    use crate::contract::sv::multitest_utils::CodeId;
    use sylvia::{
        multitest::App,
        types::{SvCustomMsg, SvCustomQuery},
    };

    #[test]
    fn proxy_methods() {
        let app = App::<cw_multi_test::BasicApp<SvCustomMsg, SvCustomQuery>>::custom(|_, _, _| {});
        let code_id = CodeId::<
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
        >::store_code(&app);

        let owner = "owner";

        let contract = code_id
            .instantiate(SvCustomMsg {})
            .with_label("GenericContract")
            .with_admin(owner)
            .call(owner)
            .unwrap();

        contract
            .custom_generic_execute_one(vec![], vec![])
            .call(owner)
            .unwrap();
        contract
            .custom_generic_execute_two(vec![], vec![])
            .call(owner)
            .unwrap();
        contract
            .custom_generic_query_one(SvCustomMsg {}, SvCustomMsg {})
            .unwrap();
        contract
            .custom_generic_query_two(SvCustomMsg {}, SvCustomMsg {})
            .unwrap();
    }
}

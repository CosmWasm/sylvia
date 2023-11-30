use cosmwasm_std::{CosmosMsg, Response, StdError, StdResult};
use custom_and_generic::CustomAndGeneric;
use serde::Deserialize;
use sylvia::contract;
use sylvia::types::{CustomMsg, CustomQuery, ExecCtx, QueryCtx, SvCustomMsg};

#[contract(module = crate::contract)]
#[messages(custom_and_generic as CustomAndGeneric)]
#[sv::custom(msg=CustomMsgT, query=CustomQueryT)]
impl<InstantiateT, Exec1T, Exec2T, Exec3T, QueryT, MigrateT, CustomMsgT, CustomQueryT, FieldT>
    CustomAndGeneric<Exec1T, Exec2T, Exec3T, QueryT, SvCustomMsg, CustomMsgT, CustomQueryT>
    for crate::contract::GenericsForwardedContract<
        InstantiateT,
        Exec1T,
        Exec2T,
        Exec3T,
        QueryT,
        MigrateT,
        CustomMsgT,
        CustomQueryT,
        FieldT,
    >
where
    for<'msg_de> InstantiateT: cosmwasm_std::CustomMsg + Deserialize<'msg_de> + 'msg_de,
    Exec1T: CustomMsg + 'static,
    Exec2T: CustomMsg + 'static,
    Exec3T: CustomMsg + 'static,
    QueryT: CustomMsg + 'static,
    MigrateT: CustomMsg + 'static,
    CustomMsgT: CustomMsg + 'static,
    CustomQueryT: CustomQuery + 'static,
    FieldT: 'static,
{
    type Error = StdError;

    #[msg(exec)]
    fn custom_generic_execute_one(
        &self,
        _ctx: ExecCtx<CustomQueryT>,
        _msgs1: Vec<CosmosMsg<Exec1T>>,
        _msgs2: Vec<CosmosMsg<Exec2T>>,
    ) -> StdResult<Response<CustomMsgT>> {
        Ok(Response::new())
    }

    #[msg(exec)]
    fn custom_generic_execute_two(
        &self,
        _ctx: ExecCtx<CustomQueryT>,
        _msgs2: Vec<CosmosMsg<Exec2T>>,
        _msgs1: Vec<CosmosMsg<Exec3T>>,
    ) -> StdResult<Response<CustomMsgT>> {
        Ok(Response::new())
    }

    #[msg(query)]
    fn custom_generic_query(
        &self,
        _ctx: QueryCtx<CustomQueryT>,
        _msg: QueryT,
    ) -> StdResult<SvCustomMsg> {
        Ok(SvCustomMsg {})
    }
}

#[cfg(test)]
mod tests {
    use super::sv::test_utils::CustomAndGeneric;
    use crate::contract::sv::multitest_utils::CodeId;
    use sylvia::multitest::App;
    use sylvia::types::{SvCustomMsg, SvCustomQuery};

    #[test]
    fn proxy_methods() {
        let app = App::<cw_multi_test::BasicApp<SvCustomMsg, SvCustomQuery>>::custom(|_, _, _| {});
        let code_id = CodeId::<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            sylvia::types::SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomQuery,
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
        contract.custom_generic_query(SvCustomMsg {}).unwrap();
    }
}

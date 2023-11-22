use cosmwasm_std::{CosmosMsg, Response, StdError, StdResult};
use custom_and_generic::CustomAndGeneric;
use serde::Deserialize;
use sylvia::contract;
use sylvia::types::{CustomMsg, CustomQuery, ExecCtx, QueryCtx, SvCustomMsg};

#[contract(module = crate::contract)]
#[messages(custom_and_generic as CustomAndGeneric)]
#[sv::custom(msg=CustomMsgT, query=CustomQueryT)]
impl<InstantiateT, ExecT, QueryT, MigrateT, CustomMsgT, CustomQueryT, FieldT>
    CustomAndGeneric<ExecT, QueryT, SvCustomMsg, CustomMsgT, CustomQueryT>
    for crate::contract::GenericsForwardedContract<
        InstantiateT,
        ExecT,
        QueryT,
        MigrateT,
        CustomMsgT,
        CustomQueryT,
        FieldT,
    >
where
    for<'msg_de> InstantiateT: cosmwasm_std::CustomMsg + Deserialize<'msg_de> + 'msg_de,
    ExecT: CustomMsg + 'static,
    QueryT: CustomMsg + 'static,
    MigrateT: CustomMsg + 'static,
    CustomMsgT: CustomMsg + 'static,
    CustomQueryT: CustomQuery + 'static,
    FieldT: 'static,
{
    type Error = StdError;

    #[msg(exec)]
    fn custom_generic_execute(
        &self,
        _ctx: ExecCtx<CustomQueryT>,
        _msgs: Vec<CosmosMsg<ExecT>>,
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

        contract.custom_generic_execute(vec![]).call(owner).unwrap();
        contract.custom_generic_query(SvCustomMsg {}).unwrap();
    }
}

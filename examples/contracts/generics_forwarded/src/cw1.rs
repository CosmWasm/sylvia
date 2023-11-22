use cosmwasm_schema::schemars::JsonSchema;
use cosmwasm_std::{CosmosMsg, CustomMsg, Response, StdError, StdResult};
use cw1::{CanExecuteResp, Cw1};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use sylvia::contract;
use sylvia::types::{CustomQuery, ExecCtx, QueryCtx};

#[contract(module = crate::contract)]
#[messages(cw1 as Cw1)]
#[sv::custom(msg=CustomMsgT, query=CustomQueryT)]
impl<InstantiateT, ExecT, QueryT, MigrateT, CustomMsgT, CustomQueryT, FieldT> Cw1
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
    for<'msg_de> InstantiateT: CustomMsg + Deserialize<'msg_de> + 'msg_de,
    ExecT: CustomMsg + DeserializeOwned + 'static,
    QueryT: CustomMsg + DeserializeOwned + 'static,
    MigrateT: CustomMsg + DeserializeOwned + 'static,
    CustomMsgT: CustomMsg + DeserializeOwned + 'static,
    CustomQueryT: CustomQuery + JsonSchema + 'static,
    FieldT: 'static,
{
    type Error = StdError;

    #[msg(exec)]
    fn execute(&self, _ctx: ExecCtx, _msgs: Vec<CosmosMsg>) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(query)]
    fn can_execute(
        &self,
        _ctx: QueryCtx,
        _sender: String,
        _msg: CosmosMsg,
    ) -> StdResult<CanExecuteResp> {
        Ok(CanExecuteResp::default())
    }
}

#[cfg(test)]
mod tests {
    use super::sv::test_utils::Cw1;
    use crate::contract::sv::multitest_utils::CodeId;
    use cosmwasm_std::{CosmosMsg, Empty};
    use sylvia::{
        multitest::App,
        types::{SvCustomMsg, SvCustomQuery},
    };

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

        contract.execute(vec![]).call(owner).unwrap();
        contract
            .can_execute("sender".to_owned(), CosmosMsg::Custom(Empty {}))
            .unwrap();
    }
}

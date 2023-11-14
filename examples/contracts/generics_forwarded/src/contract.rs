use cosmwasm_std::{Reply, Response, StdResult};
use cw_storage_plus::Item;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use sylvia::types::{
    CustomMsg, CustomQuery, ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx, ReplyCtx, SvCustomMsg,
};
use sylvia::{contract, schemars};

pub struct GenericsForwardedContract<
    InstantiateT,
    ExecT,
    QueryT,
    MigrateT,
    CustomMsgT,
    CustomQueryT,
    FieldT,
> {
    _field: Item<'static, FieldT>,
    _phantom: std::marker::PhantomData<(
        InstantiateT,
        ExecT,
        QueryT,
        MigrateT,
        CustomMsgT,
        CustomQueryT,
    )>,
}

#[contract]
#[messages(generic<ExecT, QueryT, SvCustomMsg> as Generic: custom(msg, query))]
#[sv::custom(msg=CustomMsgT, query=CustomQueryT)]
impl<InstantiateT, ExecT, QueryT, MigrateT, CustomMsgT, CustomQueryT, FieldT>
    GenericsForwardedContract<
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
    CustomQueryT: CustomQuery + 'static,
    FieldT: 'static,
{
    pub const fn new() -> Self {
        Self {
            _field: Item::new("field"),
            _phantom: std::marker::PhantomData,
        }
    }

    #[msg(instantiate)]
    pub fn instantiate(
        &self,
        _ctx: InstantiateCtx<CustomQueryT>,
        _msg: InstantiateT,
    ) -> StdResult<Response<CustomMsgT>> {
        Ok(Response::new())
    }

    #[msg(exec)]
    pub fn contract_execute(
        &self,
        _ctx: ExecCtx<CustomQueryT>,
        _msg: ExecT,
    ) -> StdResult<Response<CustomMsgT>> {
        Ok(Response::new())
    }

    #[msg(query)]
    pub fn contract_query(&self, _ctx: QueryCtx<CustomQueryT>, _msg: QueryT) -> StdResult<String> {
        Ok(String::default())
    }

    #[msg(migrate)]
    pub fn migrate(
        &self,
        _ctx: MigrateCtx<CustomQueryT>,
        _msg: MigrateT,
    ) -> StdResult<Response<CustomMsgT>> {
        Ok(Response::new())
    }

    #[allow(dead_code)]
    #[msg(reply)]
    fn reply(
        &self,
        _ctx: ReplyCtx<CustomQueryT>,
        _reply: Reply,
    ) -> StdResult<Response<CustomMsgT>> {
        Ok(Response::new())
    }
}

#[cfg(test)]
mod tests {
    use super::sv::multitest_utils::CodeId;
    use sylvia::multitest::App;
    use sylvia::types::{SvCustomMsg, SvCustomQuery};

    #[test]
    fn generic_contract() {
        let app = App::<cw_multi_test::BasicApp<SvCustomMsg, SvCustomQuery>>::custom(|_, _, _| {});
        let code_id: CodeId<
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            super::SvCustomMsg,
            super::SvCustomMsg,
            SvCustomQuery,
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

        contract.contract_execute(SvCustomMsg).call(owner).unwrap();
        contract.contract_query(SvCustomMsg).unwrap();
        contract
            .migrate(SvCustomMsg)
            .call(owner, code_id.code_id())
            .unwrap();
    }
}

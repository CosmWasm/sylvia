use cosmwasm_std::{Reply, Response, StdResult};
use cw_storage_plus::Item;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use sylvia::types::{
    CustomMsg, CustomQuery, ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx, ReplyCtx, SvCustomMsg,
    SvCustomQuery,
};
use sylvia::{contract, schemars};

#[cfg(not(feature = "library"))]
use sylvia::entry_points;

pub struct GenericContract<
    InstantiateParam,
    ExecParam,
    QueryParam,
    MigrateParam,
    RetType,
    CtxType,
    FieldType,
> {
    _field: Item<'static, FieldType>,
    _phantom: std::marker::PhantomData<(
        InstantiateParam,
        ExecParam,
        QueryParam,
        MigrateParam,
        RetType,
        CtxType,
    )>,
}

#[cfg_attr(not(feature = "library"), entry_points(generics<SvCustomMsg, SvCustomMsg, SvCustomMsg, sylvia::types::SvCustomMsg, SvCustomMsg, SvCustomQuery, String>))]
#[contract]
#[messages(cw1 as Cw1: custom(msg, query))]
#[messages(generic<SvCustomMsg, SvCustomMsg, sylvia::types::SvCustomMsg> as Generic: custom(msg, query))]
#[messages(custom_and_generic<SvCustomMsg, SvCustomMsg, SvCustomQuery, sylvia::types::SvCustomMsg> as CustomAndGeneric)]
#[sv::custom(msg=SvCustomMsg, query=SvCustomQuery)]
impl<InstantiateParam, ExecParam, QueryParam, MigrateParam, RetType, CtxQuery, FieldType>
    GenericContract<
        InstantiateParam,
        ExecParam,
        QueryParam,
        MigrateParam,
        RetType,
        CtxQuery,
        FieldType,
    >
where
    for<'msg_de> InstantiateParam: CustomMsg + Deserialize<'msg_de> + 'msg_de,
    ExecParam: CustomMsg + DeserializeOwned + 'static,
    QueryParam: CustomMsg + DeserializeOwned + 'static,
    MigrateParam: CustomMsg + DeserializeOwned + 'static,
    RetType: CustomMsg + DeserializeOwned + 'static,
    CtxQuery: CustomQuery + 'static,
    FieldType: 'static,
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
        _ctx: InstantiateCtx<SvCustomQuery>,
        _msg: InstantiateParam,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[msg(exec)]
    pub fn contract_execute(
        &self,
        _ctx: ExecCtx<SvCustomQuery>,
        _msg: ExecParam,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[msg(query)]
    pub fn contract_query(
        &self,
        _ctx: QueryCtx<SvCustomQuery>,
        _msg: QueryParam,
    ) -> StdResult<String> {
        Ok(String::default())
    }

    #[msg(migrate)]
    pub fn migrate(
        &self,
        _ctx: MigrateCtx<SvCustomQuery>,
        _msg: MigrateParam,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[allow(dead_code)]
    #[msg(reply)]
    fn reply(
        &self,
        _ctx: ReplyCtx<SvCustomQuery>,
        _reply: Reply,
    ) -> StdResult<Response<SvCustomMsg>> {
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

use cosmwasm_std::{Reply, Response, StdResult};
use cw_storage_plus::Item;
use serde::Deserialize;
use sylvia::types::{
    CustomMsg, CustomQuery, ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx, ReplyCtx, SvCustomMsg,
};
use sylvia::{contract, schemars};

pub struct GenericsForwardedContract<
    InstantiateT,
    Exec1T,
    Exec2T,
    Exec3T,
    Query1T,
    Query2T,
    Query3T,
    MigrateT,
    CustomMsgT,
    CustomQueryT,
    FieldT,
> {
    _field: Item<'static, FieldT>,
    #[allow(clippy::type_complexity)]
    _phantom: std::marker::PhantomData<(
        InstantiateT,
        Exec1T,
        Exec2T,
        Exec3T,
        Query1T,
        Query2T,
        Query3T,
        MigrateT,
        CustomMsgT,
        CustomQueryT,
    )>,
}

// TODO: Add entry points call.
#[contract]
#[messages(generic<Exec1T, Exec2T, Exec3T, Query1T, SvCustomMsg> as Generic: custom(msg, query))]
#[messages(cw1 as Cw1: custom(msg, query))]
#[messages(custom_and_generic<Exec1T, Exec2T, Exec3T, Query1T, SvCustomMsg, CustomMsgT, CustomQueryT> as CustomAndGeneric)]
#[sv::custom(msg=CustomMsgT, query=CustomQueryT)]
impl<
        InstantiateT,
        Exec1T,
        Exec2T,
        Exec3T,
        Query1T,
        Query2T,
        Query3T,
        MigrateT,
        CustomMsgT,
        CustomQueryT,
        FieldT,
    >
    GenericsForwardedContract<
        InstantiateT,
        Exec1T,
        Exec2T,
        Exec3T,
        Query1T,
        Query2T,
        Query3T,
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
    Query1T: CustomMsg + 'static,
    Query2T: CustomMsg + 'static,
    Query3T: CustomMsg + 'static,
    MigrateT: CustomMsg + 'static,
    CustomMsgT: CustomMsg + 'static,
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
    pub fn contract_execute_one(
        &self,
        _ctx: ExecCtx<CustomQueryT>,
        _msg1: Exec1T,
        _msg2: Exec2T,
    ) -> StdResult<Response<CustomMsgT>> {
        Ok(Response::new())
    }

    #[msg(exec)]
    pub fn contract_execute_two(
        &self,
        _ctx: ExecCtx<CustomQueryT>,
        _msg1: Exec2T,
        _msg2: Exec3T,
    ) -> StdResult<Response<CustomMsgT>> {
        Ok(Response::new())
    }

    #[msg(query)]
    pub fn contract_query_one(
        &self,
        _ctx: QueryCtx<CustomQueryT>,
        _msg1: Query1T,
        _msg2: Query2T,
    ) -> StdResult<String> {
        Ok(String::default())
    }

    #[msg(query)]
    pub fn contract_query_two(
        &self,
        _ctx: QueryCtx<CustomQueryT>,
        _msg1: Query2T,
        _msg2: Query3T,
    ) -> StdResult<String> {
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
            SvCustomMsg,
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

        contract
            .contract_execute_one(SvCustomMsg, SvCustomMsg)
            .call(owner)
            .unwrap();
        contract
            .contract_execute_two(SvCustomMsg, SvCustomMsg)
            .call(owner)
            .unwrap();
        contract
            .contract_query_one(SvCustomMsg, SvCustomMsg)
            .unwrap();
        contract
            .contract_query_two(SvCustomMsg, SvCustomMsg)
            .unwrap();
        contract
            .migrate(SvCustomMsg)
            .call(owner, code_id.code_id())
            .unwrap();
    }
}

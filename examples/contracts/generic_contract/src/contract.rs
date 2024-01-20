use cosmwasm_std::{Reply, Response, StdResult};
use cw_storage_plus::Item;
use serde::Deserialize;
use sylvia::types::{
    CustomMsg, ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx, ReplyCtx, SvCustomMsg, SvCustomQuery,
};
use sylvia::{contract, schemars};

#[cfg(not(feature = "library"))]
use sylvia::entry_points;

pub struct GenericContract<
    InstantiateT,
    Exec1T,
    Exec2T,
    Exec3T,
    Query1T,
    Query2T,
    Query3T,
    MigrateT,
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
        FieldT,
    )>,
}

#[cfg_attr(not(feature = "library"), entry_points(generics<SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, sylvia::types::SvCustomMsg, String>))]
#[contract]
#[messages(cw1 as Cw1: custom(msg, query))]
#[messages(generic<SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg> as Generic: custom(msg, query))]
#[messages(custom_and_generic<SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg> as CustomAndGeneric)]
#[sv::custom(msg=SvCustomMsg, query=SvCustomQuery)]
impl<InstantiateT, Exec1T, Exec2T, Exec3T, Query1T, Query2T, Query3T, MigrateT, FieldT>
    GenericContract<
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
    for<'msg_de> InstantiateT: CustomMsg + Deserialize<'msg_de> + 'msg_de,
    Exec1T: CustomMsg + 'static,
    Exec2T: CustomMsg + 'static,
    Exec3T: CustomMsg + 'static,
    Query1T: CustomMsg + 'static,
    Query2T: CustomMsg + 'static,
    Query3T: CustomMsg + 'static,
    MigrateT: CustomMsg + 'static,
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
        _ctx: InstantiateCtx<SvCustomQuery>,
        _msg: InstantiateT,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[msg(exec)]
    pub fn contract_execute_one(
        &self,
        _ctx: ExecCtx<SvCustomQuery>,
        _msg1: Exec1T,
        _msg2: Exec2T,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[msg(exec)]
    pub fn contract_execute_two(
        &self,
        _ctx: ExecCtx<SvCustomQuery>,
        _msg1: Exec2T,
        _msg2: Exec3T,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[msg(query)]
    pub fn contract_query_one(
        &self,
        _ctx: QueryCtx<SvCustomQuery>,
        _msg1: Query1T,
        _msg2: Query1T,
    ) -> StdResult<String> {
        Ok(String::default())
    }

    #[msg(query)]
    pub fn contract_query_two(
        &self,
        _ctx: QueryCtx<SvCustomQuery>,
        _msg1: Query1T,
        _msg2: Query1T,
    ) -> StdResult<String> {
        Ok(String::default())
    }

    #[msg(migrate)]
    pub fn migrate(
        &self,
        _ctx: MigrateCtx<SvCustomQuery>,
        _msg: MigrateT,
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
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            super::SvCustomMsg,
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

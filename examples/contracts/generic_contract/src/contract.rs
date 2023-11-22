use cosmwasm_std::{Reply, Response, StdResult};
use cw_storage_plus::Item;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use sylvia::types::{
    CustomMsg, ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx, ReplyCtx, SvCustomMsg, SvCustomQuery,
};
use sylvia::{contract, schemars};

#[cfg(not(feature = "library"))]
use sylvia::entry_points;

pub struct GenericContract<InstantiateT, ExecT, QueryT, MigrateT, FieldT> {
    _field: Item<'static, FieldT>,
    _phantom: std::marker::PhantomData<(InstantiateT, ExecT, QueryT, MigrateT)>,
}

#[cfg_attr(not(feature = "library"), entry_points(generics<SvCustomMsg, SvCustomMsg, SvCustomMsg, sylvia::types::SvCustomMsg, String>))]
#[contract]
#[messages(cw1 as Cw1: custom(msg, query))]
#[messages(generic<SvCustomMsg, SvCustomMsg, SvCustomMsg> as Generic: custom(msg, query))]
#[messages(custom_and_generic<SvCustomMsg, SvCustomMsg,SvCustomMsg, SvCustomQuery, sylvia::types::SvCustomMsg> as CustomAndGeneric)]
#[sv::custom(msg=SvCustomMsg, query=SvCustomQuery)]
impl<InstantiateT, ExecT, QueryT, MigrateT, FieldT>
    GenericContract<InstantiateT, ExecT, QueryT, MigrateT, FieldT>
where
    for<'msg_de> InstantiateT: CustomMsg + Deserialize<'msg_de> + 'msg_de,
    ExecT: CustomMsg + DeserializeOwned + 'static,
    QueryT: CustomMsg + DeserializeOwned + 'static,
    MigrateT: CustomMsg + DeserializeOwned + 'static,
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
    pub fn contract_execute(
        &self,
        _ctx: ExecCtx<SvCustomQuery>,
        _msg: ExecT,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[msg(query)]
    pub fn contract_query(&self, _ctx: QueryCtx<SvCustomQuery>, _msg: QueryT) -> StdResult<String> {
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
        let code_id: CodeId<SvCustomMsg, SvCustomMsg, SvCustomMsg, super::SvCustomMsg, String, _> =
            CodeId::store_code(&app);

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

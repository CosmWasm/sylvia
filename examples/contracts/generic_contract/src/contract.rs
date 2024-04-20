use cosmwasm_std::{Reply, Response, StdResult};
use cw_storage_plus::Item;
use serde::Deserialize;
use sylvia::types::{CustomMsg, ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx, ReplyCtx, SudoCtx};
use sylvia::{contract, schemars};

#[cfg(not(feature = "library"))]
use sylvia::entry_points;

#[cosmwasm_schema::cw_serde]
pub struct SvCustomMsg;
impl cosmwasm_std::CustomMsg for SvCustomMsg {}

#[cosmwasm_schema::cw_serde]
pub struct SvCustomQuery;
impl cosmwasm_std::CustomQuery for SvCustomQuery {}

pub struct GenericContract<
    InstantiateT,
    Exec1T,
    Exec2T,
    Exec3T,
    Query1T,
    Query2T,
    Query3T,
    Sudo1T,
    Sudo2T,
    Sudo3T,
    MigrateT,
    FieldT,
> {
    _field: Item<FieldT>,
    #[allow(clippy::type_complexity)]
    _phantom: std::marker::PhantomData<(
        InstantiateT,
        Exec1T,
        Exec2T,
        Exec3T,
        Query1T,
        Query2T,
        Query3T,
        Sudo1T,
        Sudo2T,
        Sudo3T,
        MigrateT,
        FieldT,
    )>,
}

#[cfg_attr(not(feature = "library"), entry_points(generics<SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, String>))]
#[contract]
#[sv::messages(cw1 as Cw1: custom(msg, query))]
#[sv::messages(generic as Generic: custom(msg, query))]
#[sv::messages(custom_and_generic as CustomAndGeneric)]
#[sv::custom(msg=SvCustomMsg, query=SvCustomQuery)]
impl<
        InstantiateT,
        Exec1T,
        Exec2T,
        Exec3T,
        Query1T,
        Query2T,
        Query3T,
        Sudo1T,
        Sudo2T,
        Sudo3T,
        MigrateT,
        FieldT,
    >
    GenericContract<
        InstantiateT,
        Exec1T,
        Exec2T,
        Exec3T,
        Query1T,
        Query2T,
        Query3T,
        Sudo1T,
        Sudo2T,
        Sudo3T,
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
    Sudo1T: CustomMsg + 'static,
    Sudo2T: CustomMsg + 'static,
    Sudo3T: CustomMsg + 'static,
    MigrateT: CustomMsg + 'static,
    FieldT: 'static,
{
    pub const fn new() -> Self {
        Self {
            _field: Item::new("field"),
            _phantom: std::marker::PhantomData,
        }
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(
        &self,
        _ctx: InstantiateCtx<SvCustomQuery>,
        _msg: InstantiateT,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[sv::msg(exec)]
    pub fn contract_execute_one(
        &self,
        _ctx: ExecCtx<SvCustomQuery>,
        _msg1: Exec1T,
        _msg2: Exec2T,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[sv::msg(exec)]
    pub fn contract_execute_two(
        &self,
        _ctx: ExecCtx<SvCustomQuery>,
        _msg1: Exec2T,
        _msg2: Exec3T,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[sv::msg(query)]
    pub fn contract_query_one(
        &self,
        _ctx: QueryCtx<SvCustomQuery>,
        _msg1: Query1T,
        _msg2: Query1T,
    ) -> StdResult<String> {
        Ok(String::default())
    }

    #[sv::msg(query)]
    pub fn contract_query_two(
        &self,
        _ctx: QueryCtx<SvCustomQuery>,
        _msg1: Query1T,
        _msg2: Query1T,
    ) -> StdResult<String> {
        Ok(String::default())
    }

    #[sv::msg(sudo)]
    fn contract_sudo_one(
        &self,
        _ctx: SudoCtx<SvCustomQuery>,
        _msgs1: Sudo1T,
        _msgs2: Sudo2T,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[sv::msg(sudo)]
    fn contract_sudo_two(
        &self,
        _ctx: SudoCtx<SvCustomQuery>,
        _msgs1: Sudo2T,
        _msgs2: Sudo3T,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[sv::msg(migrate)]
    pub fn migrate(
        &self,
        _ctx: MigrateCtx<SvCustomQuery>,
        _msg: MigrateT,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }

    #[allow(dead_code)]
    #[sv::msg(reply)]
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
    use super::sv::mt::CodeId;
    use super::{GenericContract, SvCustomMsg, SvCustomQuery};
    use crate::contract::sv::mt::GenericContractProxy;
    use cw_multi_test::IntoBech32;
    use sylvia::multitest::App;

    #[test]
    fn generic_contract() {
        let app = App::<cw_multi_test::BasicApp<SvCustomMsg, SvCustomQuery>>::custom(|_, _, _| {});
        #[allow(clippy::type_complexity)]
        let code_id: CodeId<
            GenericContract<
                SvCustomMsg,
                SvCustomMsg,
                SvCustomMsg,
                SvCustomMsg,
                SvCustomMsg,
                SvCustomMsg,
                SvCustomMsg,
                SvCustomMsg,
                SvCustomMsg,
                SvCustomMsg,
                SvCustomMsg,
                String,
            >,
            _,
        > = CodeId::store_code(&app);

        let owner = "owner".into_bech32();

        let contract = code_id
            .instantiate(SvCustomMsg {})
            .with_label("GenericContract")
            .with_admin(owner.as_str())
            .call(&owner)
            .unwrap();

        contract
            .contract_execute_one(SvCustomMsg, SvCustomMsg)
            .call(&owner)
            .unwrap();
        contract
            .contract_execute_two(SvCustomMsg, SvCustomMsg)
            .call(&owner)
            .unwrap();
        contract
            .contract_query_one(SvCustomMsg, SvCustomMsg)
            .unwrap();
        contract
            .contract_query_two(SvCustomMsg, SvCustomMsg)
            .unwrap();
        contract
            .contract_sudo_one(SvCustomMsg, SvCustomMsg)
            .unwrap();
        contract
            .contract_sudo_two(SvCustomMsg, SvCustomMsg)
            .unwrap();
        contract
            .migrate(SvCustomMsg)
            .call(&owner, code_id.code_id())
            .unwrap();
    }
}

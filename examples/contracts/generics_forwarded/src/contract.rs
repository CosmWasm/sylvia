use crate::error::ContractError;
use cw_storage_plus::Item;
use serde::Deserialize;
use sylvia::cw_std::{Reply, Response};
use sylvia::types::{
    CustomMsg, CustomQuery, ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx, ReplyCtx, SudoCtx,
};
use sylvia::{contract, schemars};

#[cosmwasm_schema::cw_serde]
pub struct SvCustomMsg;
impl sylvia::cw_std::CustomMsg for SvCustomMsg {}

#[cosmwasm_schema::cw_serde]
pub struct SvCustomQuery;
impl sylvia::cw_std::CustomQuery for SvCustomQuery {}

pub struct GenericsForwardedContract<
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
    CustomMsgT,
    CustomQueryT,
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
        CustomMsgT,
        CustomQueryT,
    )>,
}

#[cfg_attr(not(feature = "library"), sylvia::entry_points(generics<SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomQuery, String>))]
#[contract]
#[sv::error(ContractError)]
#[sv::messages(generic as Generic: custom(msg, query))]
#[sv::messages(cw1 as Cw1: custom(msg, query))]
#[sv::messages(custom_and_generic as CustomAndGeneric)]
#[sv::custom(msg=CustomMsgT, query=CustomQueryT)]
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
        Sudo1T,
        Sudo2T,
        Sudo3T,
        MigrateT,
        CustomMsgT,
        CustomQueryT,
        FieldT,
    >
where
    for<'msg_de> InstantiateT: sylvia::cw_std::CustomMsg + Deserialize<'msg_de> + 'msg_de,
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

    #[sv::msg(instantiate)]
    pub fn instantiate(
        &self,
        _ctx: InstantiateCtx<CustomQueryT>,
        _msg: InstantiateT,
    ) -> Result<Response<CustomMsgT>, ContractError> {
        Ok(Response::new())
    }

    #[sv::msg(exec)]
    pub fn contract_execute_one(
        &self,
        _ctx: ExecCtx<CustomQueryT>,
        _msg1: Exec1T,
        _msg2: Exec2T,
    ) -> Result<Response<CustomMsgT>, ContractError> {
        Ok(Response::new())
    }

    #[sv::msg(exec)]
    pub fn contract_execute_two(
        &self,
        _ctx: ExecCtx<CustomQueryT>,
        _msg1: Exec2T,
        _msg2: Exec3T,
    ) -> Result<Response<CustomMsgT>, ContractError> {
        Ok(Response::new())
    }

    #[sv::msg(query)]
    pub fn contract_query_one(
        &self,
        _ctx: QueryCtx<CustomQueryT>,
        _msg1: Query1T,
        _msg2: Query2T,
    ) -> Result<String, ContractError> {
        Ok(String::default())
    }

    #[sv::msg(query)]
    pub fn contract_query_two(
        &self,
        _ctx: QueryCtx<CustomQueryT>,
        _msg1: Query2T,
        _msg2: Query3T,
    ) -> Result<String, ContractError> {
        Ok(String::default())
    }

    #[sv::msg(sudo)]
    fn contract_sudo_one(
        &self,
        _ctx: SudoCtx<CustomQueryT>,
        _msgs1: Sudo1T,
        _msgs2: Sudo2T,
    ) -> Result<Response<CustomMsgT>, ContractError> {
        Ok(Response::new())
    }

    #[sv::msg(sudo)]
    fn contract_sudo_two(
        &self,
        _ctx: SudoCtx<CustomQueryT>,
        _msgs1: Sudo2T,
        _msgs2: Sudo3T,
    ) -> Result<Response<CustomMsgT>, ContractError> {
        Ok(Response::new())
    }

    #[sv::msg(migrate)]
    pub fn migrate(
        &self,
        _ctx: MigrateCtx<CustomQueryT>,
        _msg: MigrateT,
    ) -> Result<Response<CustomMsgT>, ContractError> {
        Ok(Response::new())
    }

    #[allow(dead_code)]
    #[sv::msg(reply)]
    fn reply(
        &self,
        _ctx: ReplyCtx<CustomQueryT>,
        _reply: Reply,
    ) -> Result<Response<CustomMsgT>, ContractError> {
        Ok(Response::new())
    }
}

#[cfg(test)]
mod tests {
    use super::sv::mt::CodeId;
    use super::{GenericsForwardedContract, SvCustomMsg, SvCustomQuery};
    use crate::contract::sv::mt::GenericsForwardedContractProxy;
    use cw_multi_test::IntoBech32;
    use sylvia::multitest::App;

    #[test]
    fn generic_contract() {
        let app = App::<cw_multi_test::BasicApp<SvCustomMsg, SvCustomQuery>>::custom(|_, _, _| {});
        #[allow(clippy::type_complexity)]
        let code_id: CodeId<
            GenericsForwardedContract<
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
                SvCustomMsg,
                SvCustomQuery,
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

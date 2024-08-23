use custom_and_generic::CustomAndGeneric;
use serde::Deserialize;
use sylvia::cw_std::{CosmosMsg, Response};
use sylvia::types::{CustomMsg, CustomQuery, ExecCtx, QueryCtx, SudoCtx};

use crate::contract::{GenericsForwardedContract, SvCustomMsg};
use crate::error::ContractError;

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
    > CustomAndGeneric
    for GenericsForwardedContract<
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
    type Error = ContractError;
    type Exec1T = Exec1T;
    type Exec2T = Exec2T;
    type Exec3T = Exec3T;
    type Query1T = Query1T;
    type Query2T = Query2T;
    type Query3T = Query3T;
    type Sudo1T = Sudo1T;
    type Sudo2T = Sudo2T;
    type Sudo3T = Sudo3T;
    type ExecC = CustomMsgT;
    type QueryC = CustomQueryT;
    type RetT = SvCustomMsg;

    fn custom_generic_execute_one(
        &self,
        _ctx: ExecCtx<Self::QueryC>,
        _msgs1: Vec<CosmosMsg<Self::Exec1T>>,
        _msgs2: Vec<CosmosMsg<Self::Exec2T>>,
    ) -> Result<Response<Self::ExecC>, ContractError> {
        Ok(Response::new())
    }

    fn custom_generic_execute_two(
        &self,
        _ctx: ExecCtx<Self::QueryC>,
        _msgs2: Vec<CosmosMsg<Self::Exec2T>>,
        _msgs1: Vec<CosmosMsg<Self::Exec3T>>,
    ) -> Result<Response<Self::ExecC>, ContractError> {
        Ok(Response::new())
    }

    fn custom_generic_query_one(
        &self,
        _ctx: QueryCtx<Self::QueryC>,
        _msg1: Self::Query1T,
        _msg2: Self::Query2T,
    ) -> Result<SvCustomMsg, ContractError> {
        Ok(SvCustomMsg {})
    }

    fn custom_generic_query_two(
        &self,
        _ctx: QueryCtx<Self::QueryC>,
        _msg1: Self::Query2T,
        _msg2: Self::Query3T,
    ) -> Result<SvCustomMsg, ContractError> {
        Ok(SvCustomMsg {})
    }

    fn custom_generic_sudo_one(
        &self,
        _ctx: SudoCtx<Self::QueryC>,
        _msgs1: CosmosMsg<Self::Sudo1T>,
        _msgs2: CosmosMsg<Self::Sudo2T>,
    ) -> Result<Response<Self::ExecC>, ContractError> {
        Ok(Response::new())
    }

    fn custom_generic_sudo_two(
        &self,
        _ctx: SudoCtx<Self::QueryC>,
        _msgs1: CosmosMsg<Self::Sudo2T>,
        _msgs2: CosmosMsg<Self::Sudo3T>,
    ) -> Result<Response<Self::ExecC>, ContractError> {
        Ok(Response::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::contract::sv::mt::CodeId;
    use crate::contract::{GenericsForwardedContract, SvCustomMsg, SvCustomQuery};
    use custom_and_generic::sv::mt::CustomAndGenericProxy;
    use cw_multi_test::IntoBech32;
    use sylvia::cw_std::CosmosMsg;
    use sylvia::multitest::App;

    #[test]
    fn proxy_methods() {
        let app = App::<cw_multi_test::BasicApp<SvCustomMsg, SvCustomQuery>>::custom(|_, _, _| {});
        let code_id = CodeId::<
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
        >::store_code(&app);

        let owner = "owner".into_bech32();

        let contract = code_id
            .instantiate(SvCustomMsg {})
            .with_label("GenericContract")
            .with_admin(owner.as_str())
            .call(&owner)
            .unwrap();

        contract
            .custom_generic_execute_one(vec![], vec![])
            .call(&owner)
            .unwrap();
        contract
            .custom_generic_execute_two(vec![], vec![])
            .call(&owner)
            .unwrap();
        contract
            .custom_generic_query_one(SvCustomMsg {}, SvCustomMsg {})
            .unwrap();
        contract
            .custom_generic_query_two(SvCustomMsg {}, SvCustomMsg {})
            .unwrap();
        contract
            .custom_generic_sudo_one(
                CosmosMsg::Custom(SvCustomMsg {}),
                CosmosMsg::Custom(SvCustomMsg {}),
            )
            .unwrap();
        contract
            .custom_generic_sudo_one(
                CosmosMsg::Custom(SvCustomMsg {}),
                CosmosMsg::Custom(SvCustomMsg {}),
            )
            .unwrap();
    }
}

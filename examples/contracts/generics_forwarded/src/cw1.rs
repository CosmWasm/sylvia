use cw1::{CanExecuteResp, Cw1};
use sylvia::ctx::{ExecCtx, QueryCtx};
use sylvia::cw_schema::schemars::JsonSchema;
use sylvia::cw_std::{CosmosMsg, CustomMsg, Empty, Response, StdResult};
use sylvia::serde::de::DeserializeOwned;
use sylvia::serde::Deserialize;
use sylvia::types::CustomQuery;

use crate::contract::GenericsForwardedContract;
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
    > Cw1
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
    for<'msg_de> InstantiateT: CustomMsg + Deserialize<'msg_de> + 'msg_de,
    Exec1T: CustomMsg + DeserializeOwned + 'static,
    Exec2T: CustomMsg + DeserializeOwned + 'static,
    Exec3T: CustomMsg + DeserializeOwned + 'static,
    Query1T: sylvia::types::CustomMsg + 'static,
    Query2T: sylvia::types::CustomMsg + 'static,
    Query3T: sylvia::types::CustomMsg + 'static,
    Sudo1T: sylvia::types::CustomMsg + 'static,
    Sudo2T: sylvia::types::CustomMsg + 'static,
    Sudo3T: sylvia::types::CustomMsg + 'static,
    MigrateT: CustomMsg + DeserializeOwned + 'static,
    CustomMsgT: CustomMsg + DeserializeOwned + 'static,
    CustomQueryT: CustomQuery + JsonSchema + 'static,
    FieldT: 'static,
{
    type Error = ContractError;
    type ExecC = Empty;
    type QueryC = Empty;

    fn execute(&self, _ctx: ExecCtx, _msgs: Vec<CosmosMsg>) -> Result<Response, Self::Error> {
        Ok(Response::new())
    }

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
    use crate::contract::sv::mt::CodeId;
    use crate::contract::{GenericsForwardedContract, SvCustomMsg, SvCustomQuery};
    use cw1::sv::mt::Cw1Proxy;
    use sylvia::cw_multi_test::{BasicApp, IntoBech32};
    use sylvia::cw_std::{CosmosMsg, Empty};
    use sylvia::multitest::App;

    #[test]
    fn proxy_methods() {
        let app = App::<BasicApp<SvCustomMsg, SvCustomQuery>>::custom(|_, _, _| {});
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

        contract.execute(vec![]).call(&owner).unwrap();
        contract
            .can_execute("sender".to_owned(), CosmosMsg::Custom(Empty {}))
            .unwrap();
    }
}

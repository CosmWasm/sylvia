use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Reply, Response, StdResult};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use sylvia::types::{CustomMsg, ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx, ReplyCtx};
use sylvia::{contract, schemars};

#[cw_serde]
pub struct ExternalMsg;
impl cosmwasm_std::CustomMsg for ExternalMsg {}

pub struct GenericContract<InstantiateParam, ExecParam, QueryParam, MigrateParam, RetType>(
    std::marker::PhantomData<(
        InstantiateParam,
        ExecParam,
        QueryParam,
        MigrateParam,
        RetType,
    )>,
);

#[contract]
impl<InstantiateParam, ExecParam, QueryParam, MigrateParam, RetType>
    GenericContract<InstantiateParam, ExecParam, QueryParam, MigrateParam, RetType>
where
    for<'msg_de> InstantiateParam: CustomMsg + Deserialize<'msg_de> + 'msg_de,
    ExecParam: CustomMsg + DeserializeOwned + 'static,
    QueryParam: CustomMsg + DeserializeOwned + 'static,
    MigrateParam: CustomMsg + DeserializeOwned + 'static,
    RetType: CustomMsg + DeserializeOwned + 'static,
{
    pub const fn new() -> Self {
        Self(std::marker::PhantomData)
    }

    #[msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx, _msg: InstantiateParam) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(exec)]
    pub fn execute(&self, _ctx: ExecCtx, _msg: ExecParam) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(query)]
    pub fn query(&self, _ctx: QueryCtx, _msg: QueryParam) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(migrate)]
    pub fn migrate(&self, _ctx: MigrateCtx, _msg: MigrateParam) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[allow(dead_code)]
    #[msg(reply)]
    fn reply(&self, _ctx: ReplyCtx, _reply: Reply) -> StdResult<Response> {
        Ok(Response::new())
    }
}

#[cfg(test)]
mod tests {
    use sylvia::multitest::App;

    use crate::contract::ExternalMsg;

    #[test]
    fn generic_contract() {
        use super::multitest_utils::CodeId;
        let app = App::default();
        let code_id: CodeId<
            ExternalMsg,
            ExternalMsg,
            ExternalMsg,
            super::ExternalMsg,
            super::ExternalMsg,
            _,
        > = CodeId::store_code(&app);

        let owner = "owner";

        let contract = code_id
            .instantiate(ExternalMsg {})
            .with_label("GenericContract")
            .with_admin(owner)
            .call(owner)
            .unwrap();

        contract.execute(ExternalMsg).call(owner).unwrap();
        contract.query(ExternalMsg).unwrap();
        contract
            .migrate(ExternalMsg)
            .call(owner, code_id.code_id())
            .unwrap();
    }
}

use cosmwasm_std::{CosmosMsg, Response, StdError, StdResult};
use generic::Generic;
use sylvia::contract;
use sylvia::types::{ExecCtx, QueryCtx, SvCustomMsg};

#[contract(module = crate::contract)]
#[messages(generic as Generic)]
#[sv::custom(msg=SvCustomMsg)]
impl<InstantiateParam, ExecParam, QueryParam, MigrateParam, RetType>
    Generic<SvCustomMsg, SvCustomMsg, sylvia::types::SvCustomMsg>
    for crate::contract::GenericContract<
        InstantiateParam,
        ExecParam,
        QueryParam,
        MigrateParam,
        RetType,
    >
{
    type Error = StdError;

    #[msg(exec)]
    fn generic_exec(
        &self,
        _ctx: ExecCtx,
        _msgs: Vec<CosmosMsg<sylvia::types::SvCustomMsg>>,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    // Sylvia will fail if single type is used to match against two different generics
    // It's because we have to map unique generics used as they can be used multiple times.
    // If for some reason like here one type would be used in place of two generics either full
    // path or some alias has to be used.
    #[msg(query)]
    fn generic_query(
        &self,
        _ctx: QueryCtx,
        _msg: sylvia::types::SvCustomMsg,
    ) -> StdResult<SvCustomMsg> {
        Ok(SvCustomMsg {})
    }
}

#[cfg(test)]
mod tests {
    use super::sv::test_utils::Generic;
    use crate::contract::sv::multitest_utils::CodeId;
    use cosmwasm_std::CosmosMsg;
    use sylvia::multitest::App;
    use sylvia::types::SvCustomMsg;

    #[test]
    fn proxy_methods() {
        let app = App::<cw_multi_test::BasicApp<SvCustomMsg>>::custom(|_, _, _| {});
        let code_id: CodeId<
            SvCustomMsg,
            sylvia::types::SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            sylvia::types::SvCustomMsg,
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
            .generic_proxy()
            .generic_exec(vec![CosmosMsg::Custom(SvCustomMsg {})])
            .call(owner)
            .unwrap();
        contract.generic_proxy().generic_query(SvCustomMsg).unwrap();
    }
}

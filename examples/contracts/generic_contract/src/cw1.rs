use cosmwasm_std::{CosmosMsg, Response, StdError, StdResult};
use cw1::{CanExecuteResp, Cw1};
use sylvia::contract;
use sylvia::types::{ExecCtx, QueryCtx};

#[contract(module = crate::contract)]
#[messages(cw1 as Cw1)]
#[sv::custom(msg=sylvia::types::SvCustomMsg, query=sylvia::types::SvCustomQuery)]
impl<InstantiateT, ExecT, QueryT, MigrateT, FieldT> Cw1
    for crate::contract::GenericContract<InstantiateT, ExecT, QueryT, MigrateT, FieldT>
{
    type Error = StdError;

    #[msg(exec)]
    fn execute(&self, _ctx: ExecCtx, _msgs: Vec<CosmosMsg>) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(query)]
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
    use super::sv::test_utils::Cw1;
    use crate::contract::sv::multitest_utils::CodeId;
    use cosmwasm_std::{CosmosMsg, Empty};
    use sylvia::{
        multitest::App,
        types::{SvCustomMsg, SvCustomQuery},
    };

    #[test]
    fn proxy_methods() {
        let app = App::<cw_multi_test::BasicApp<SvCustomMsg, SvCustomQuery>>::custom(|_, _, _| {});
        let code_id = CodeId::<
            SvCustomMsg,
            sylvia::types::SvCustomMsg,
            SvCustomMsg,
            SvCustomMsg,
            String,
            _,
        >::store_code(&app);

        let owner = "owner";

        let contract = code_id
            .instantiate(SvCustomMsg {})
            .with_label("GenericContract")
            .with_admin(owner)
            .call(owner)
            .unwrap();

        contract.execute(vec![]).call(owner).unwrap();
        contract
            .can_execute("sender".to_owned(), CosmosMsg::Custom(Empty {}))
            .unwrap();
    }
}

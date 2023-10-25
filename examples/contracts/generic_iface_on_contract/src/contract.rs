use cosmwasm_std::{Response, StdResult};
use sylvia::types::{InstantiateCtx, SvCustomMsg};
use sylvia::{contract, schemars};

pub struct NonGenericContract;

#[contract]
#[messages(generic<SvCustomMsg, sylvia::types::SvCustomMsg, SvCustomMsg> as Generic: custom(msg))]
#[messages(custom_and_generic<SvCustomMsg, SvCustomMsg, sylvia::types::SvCustomMsg> as CustomAndGeneric)]
#[messages(cw1 as Cw1: custom(msg))]
/// Required if interface returns generic `Response`
#[sv::custom(msg=SvCustomMsg)]
impl NonGenericContract {
    pub const fn new() -> Self {
        Self
    }

    #[msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{CosmosMsg, Empty};
    use sylvia::{multitest::App, types::SvCustomMsg};

    use super::NonGenericContract;
    use crate::custom_and_generic::sv::test_utils::CustomAndGeneric;
    use crate::cw1::sv::test_utils::Cw1;
    use crate::generic::sv::test_utils::Generic;

    #[test]
    fn mt_helpers() {
        let _ = NonGenericContract::new();
        let app = App::<cw_multi_test::BasicApp<SvCustomMsg>>::custom(|_, _, _| {});
        let code_id = super::sv::multitest_utils::CodeId::store_code(&app);

        let owner = "owner";

        let contract = code_id
            .instantiate()
            .with_label("Cw1Contract")
            .call(owner)
            .unwrap();

        // Non custom non generic interface
        contract
            .cw1_proxy()
            .can_execute("sender".to_owned(), CosmosMsg::Custom(Empty {}))
            .unwrap();
        contract
            .cw1_proxy()
            .execute(vec![CosmosMsg::Custom(Empty {})])
            .call(owner)
            .unwrap();

        // Non-Custom generic Interface
        contract
            .generic_proxy()
            .generic_query(SvCustomMsg {})
            .unwrap();
        contract
            .generic_proxy()
            .generic_exec(vec![CosmosMsg::Custom(SvCustomMsg {})])
            .call(owner)
            .unwrap();

        // Custom generic Interface
        contract
            .custom_and_generic_proxy()
            .custom_generic_query(SvCustomMsg {})
            .unwrap();
        contract
            .custom_and_generic_proxy()
            .custom_generic_execute(vec![CosmosMsg::Custom(SvCustomMsg {})])
            .call(owner)
            .unwrap();
    }
}

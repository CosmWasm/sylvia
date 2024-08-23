use sylvia::cw_std::{Response, StdResult};
use sylvia::types::InstantiateCtx;
use sylvia::{contract, schemars};

#[cfg(not(feature = "library"))]
use sylvia::entry_points;

pub struct NonGenericContract;

#[cosmwasm_schema::cw_serde]
pub struct SvCustomMsg;
impl sylvia::cw_std::CustomMsg for SvCustomMsg {}

#[cosmwasm_schema::cw_serde]
pub struct SvCustomQuery;
impl sylvia::cw_std::CustomQuery for SvCustomQuery {}

#[cfg_attr(not(feature = "library"), entry_points)]
#[contract]
#[sv::messages(generic as Generic: custom(msg, query))]
#[sv::messages(custom_and_generic as CustomAndGeneric)]
#[sv::messages(cw1 as Cw1: custom(msg, query))]
/// Required if interface returns generic `Response`
#[sv::custom(msg=SvCustomMsg, query=SvCustomQuery)]
impl NonGenericContract {
    pub const fn new() -> Self {
        Self
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(
        &self,
        _ctx: InstantiateCtx<SvCustomQuery>,
    ) -> StdResult<Response<SvCustomMsg>> {
        Ok(Response::new())
    }
}

#[cfg(test)]
mod tests {
    use super::{SvCustomMsg, SvCustomQuery};
    use cw_multi_test::IntoBech32;
    use sylvia::cw_std::{CosmosMsg, Empty};
    use sylvia::multitest::App;

    use super::NonGenericContract;
    use custom_and_generic::sv::mt::CustomAndGenericProxy;
    use cw1::sv::mt::Cw1Proxy;
    use generic::sv::mt::GenericProxy;

    #[test]
    fn mt_helpers() {
        let _ = NonGenericContract::new();
        let app = App::<cw_multi_test::BasicApp<SvCustomMsg, SvCustomQuery>>::custom(|_, _, _| {});
        let code_id = super::sv::mt::CodeId::store_code(&app);

        let owner = "owner".into_bech32();

        let contract = code_id
            .instantiate()
            .with_label("Cw1Contract")
            .call(&owner)
            .unwrap();

        // Non custom non generic interface
        contract
            .can_execute("sender".to_owned(), CosmosMsg::Custom(Empty {}))
            .unwrap();
        contract
            .execute(vec![CosmosMsg::Custom(Empty {})])
            .call(&owner)
            .unwrap();

        // Non-Custom generic Interface
        contract
            .generic_query_one(SvCustomMsg {}, SvCustomMsg {})
            .unwrap();
        contract
            .generic_query_two(SvCustomMsg {}, SvCustomMsg {})
            .unwrap();
        contract
            .generic_exec_one(
                vec![CosmosMsg::Custom(SvCustomMsg {})],
                vec![CosmosMsg::Custom(SvCustomMsg {})],
            )
            .call(&owner)
            .unwrap();
        contract
            .generic_exec_two(
                vec![CosmosMsg::Custom(SvCustomMsg {})],
                vec![CosmosMsg::Custom(SvCustomMsg {})],
            )
            .call(&owner)
            .unwrap();
        contract
            .generic_sudo_one(
                CosmosMsg::Custom(SvCustomMsg {}),
                CosmosMsg::Custom(SvCustomMsg {}),
            )
            .unwrap();
        contract
            .generic_sudo_two(
                CosmosMsg::Custom(SvCustomMsg {}),
                CosmosMsg::Custom(SvCustomMsg {}),
            )
            .unwrap();

        // Custom generic Interface
        contract
            .custom_generic_query_one(SvCustomMsg {}, SvCustomMsg {})
            .unwrap();
        contract
            .custom_generic_query_two(SvCustomMsg {}, SvCustomMsg {})
            .unwrap();
        contract
            .custom_generic_execute_one(
                vec![CosmosMsg::Custom(SvCustomMsg {})],
                vec![CosmosMsg::Custom(SvCustomMsg {})],
            )
            .call(&owner)
            .unwrap();
        contract
            .custom_generic_execute_two(
                vec![CosmosMsg::Custom(SvCustomMsg {})],
                vec![CosmosMsg::Custom(SvCustomMsg {})],
            )
            .call(&owner)
            .unwrap();
        contract
            .custom_generic_sudo_one(
                CosmosMsg::Custom(SvCustomMsg {}),
                CosmosMsg::Custom(SvCustomMsg {}),
            )
            .unwrap();
        contract
            .custom_generic_sudo_two(
                CosmosMsg::Custom(SvCustomMsg {}),
                CosmosMsg::Custom(SvCustomMsg {}),
            )
            .unwrap();
    }
}

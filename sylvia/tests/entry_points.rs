use cosmwasm_std::Coin;
use cw_storage_plus::Item;

#[derive(
    serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema,
)]
pub struct CountResponse {
    pub count: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    MoveFunds {
        recipient: String,
        amount: Vec<Coin>,
    },
}

pub mod migrate {
    #[derive(
        serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema,
    )]
    #[serde(rename_all = "snake_case")]
    pub struct MigrateMsg {}
}

const COUNTER: Item<u32> = Item::new("counter");

pub mod entry_points {
    use cosmwasm_std::{entry_point, DepsMut, Env, Response, StdResult};

    use crate::migrate::MigrateMsg;
    use crate::{SudoMsg, COUNTER};

    #[entry_point]
    pub fn sudo(deps: DepsMut, _env: Env, _msg: SudoMsg) -> StdResult<Response> {
        COUNTER.save(deps.storage, &3)?;
        Ok(Response::new())
    }

    #[entry_point]
    pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
        COUNTER.save(deps.storage, &5)?;
        Ok(Response::new())
    }
}

mod contract {
    use cosmwasm_std::{Response, StdResult};
    use sylvia::contract;
    use sylvia::types::{InstantiateCtx, MigrateCtx, QueryCtx};

    use crate::{CountResponse, COUNTER};

    pub struct Contract {}

    #[cfg(not(tarpaulin_include))]
    #[contract]
    #[sv::override_entry_point(sudo=crate::entry_points::sudo(crate::SudoMsg))]
    #[sv::override_entry_point(migrate=crate::entry_points::migrate(crate::migrate::MigrateMsg))]
    #[allow(dead_code)]
    impl Contract {
        #[allow(clippy::new_without_default)]
        #[allow(dead_code)]
        pub fn new() -> Self {
            Self {}
        }

        #[msg(instantiate)]
        pub fn instantiate(&self, ctx: InstantiateCtx) -> StdResult<Response> {
            COUNTER.save(ctx.deps.storage, &0)?;
            Ok(Response::new())
        }

        #[msg(migrate)]
        pub fn migrate(&self, _ctx: MigrateCtx) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[msg(query)]
        pub fn count(&self, ctx: QueryCtx) -> StdResult<CountResponse> {
            let count = COUNTER.load(ctx.deps.storage)?;
            Ok(CountResponse { count })
        }
    }
}

#[cfg(all(test, feature = "mt"))]
mod tests {
    use sylvia::multitest::App;

    use crate::contract::multitest_utils::CodeId;
    use crate::SudoMsg;

    #[test]
    fn overriden_entry_points_in_mt() {
        let app = App::default();
        let code_id = CodeId::store_code(&app);

        let owner = "owner";

        let contract = code_id
            .instantiate()
            .with_label("Contract")
            .with_admin(Some(owner))
            .call(owner)
            .unwrap();

        let count = contract.count().unwrap().count;
        assert_eq!(count, 0);

        let msg = SudoMsg::MoveFunds {
            recipient: "recipient".to_string(),
            amount: vec![],
        };

        contract
            .app
            .app_mut()
            .wasm_sudo(contract.contract_addr.clone(), &msg)
            .unwrap();

        let count = contract.count().unwrap().count;
        assert_eq!(count, 3);

        contract.migrate().call(owner, code_id.code_id()).unwrap();
        let count = contract.count().unwrap().count;
        assert_eq!(count, 5);
    }
}

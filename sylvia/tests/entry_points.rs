use contract::ContractExecMsg;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;
use cw_storage_plus::Item;

#[cw_serde]
pub struct CountResponse {
    pub count: u32,
}

#[cw_serde]
pub enum SudoMsg {
    MoveFunds {
        recipient: String,
        amount: Vec<Coin>,
    },
}

pub mod migrate {
    use cosmwasm_schema::cw_serde;

    #[cw_serde]
    pub struct MigrateMsg {}
}

pub mod exec {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, StdResult};
    use sylvia::types::ExecCtx;

    #[cw_serde]
    pub enum UserExecMsg {
        IncreaseByOne {},
    }

    pub fn increase_by_one(ctx: ExecCtx) -> StdResult<Response> {
        crate::COUNTER.update(ctx.deps.storage, |count| -> Result<u32, StdError> {
            Ok(count + 1)
        })?;
        Ok(Response::new())
    }

    #[cw_serde]
    pub enum CustomExecMsg {
        ContractExec(crate::ContractExecMsg),
        CustomExec(UserExecMsg),
    }

    impl CustomExecMsg {
        pub fn dispatch(self, ctx: (DepsMut, Env, MessageInfo)) -> StdResult<Response> {
            match self {
                CustomExecMsg::ContractExec(msg) => {
                    msg.dispatch(&crate::contract::Contract::new(), ctx)
                }
                CustomExecMsg::CustomExec(_) => increase_by_one(ctx.into()),
            }
        }
    }
}

const COUNTER: Item<u32> = Item::new("counter");

pub mod entry_points {
    use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response, StdResult};

    use crate::exec::CustomExecMsg;
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

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: CustomExecMsg,
    ) -> StdResult<Response> {
        msg.dispatch((deps, env, info))
    }
}

mod contract {
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::contract;
    use sylvia::types::{ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx};

    use crate::{CountResponse, COUNTER};

    pub struct Contract {}

    #[cfg(not(tarpaulin_include))]
    #[contract]
    #[sv::override_entry_point(sudo=crate::entry_points::sudo(crate::SudoMsg))]
    #[sv::override_entry_point(migrate=crate::entry_points::migrate(crate::migrate::MigrateMsg))]
    #[sv::override_entry_point(exec=crate::entry_points::execute(crate::exec::CustomExecMsg))]
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

        #[msg(exec)]
        pub fn increase_by_two(&self, ctx: ExecCtx) -> StdResult<Response> {
            crate::COUNTER.update(ctx.deps.storage, |count| -> Result<u32, StdError> {
                Ok(count + 2)
            })?;
            Ok(Response::new())
        }
    }
}

#[cfg(all(test, feature = "mt"))]
mod tests {
    use cosmwasm_std::Addr;
    use cw_multi_test::Executor;
    use sylvia::multitest::App;

    use crate::contract::multitest_utils::CodeId;
    use crate::contract::{ContractExecMsg, ExecMsg};
    use crate::exec::{CustomExecMsg, UserExecMsg};
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

        // custom ExecMsg
        let msg = CustomExecMsg::CustomExec(UserExecMsg::IncreaseByOne {});
        (*contract.app)
            .app_mut()
            .execute_contract(
                Addr::unchecked(owner),
                contract.contract_addr.clone(),
                &msg,
                &[],
            )
            .unwrap();

        let count = contract.count().unwrap().count;
        assert_eq!(count, 6);

        // custom ExecMsg
        let msg =
            CustomExecMsg::ContractExec(ContractExecMsg::Contract(ExecMsg::increase_by_two()));
        (*contract.app)
            .app_mut()
            .execute_contract(
                Addr::unchecked(owner),
                contract.contract_addr.clone(),
                &msg,
                &[],
            )
            .unwrap();

        let count = contract.count().unwrap().count;
        assert_eq!(count, 8);
    }
}

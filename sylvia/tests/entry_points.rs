use contract::sv::ContractExecMsg;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct CountResponse {
    pub count: u32,
}

pub mod sudo {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{Coin, DepsMut, Env, Response, StdError, StdResult};
    use sylvia::types::SudoCtx;

    #[cw_serde]
    pub enum SudoMsg {
        MoveFunds {
            recipient: String,
            amount: Vec<Coin>,
        },
    }

    impl SudoMsg {
        pub fn dispatch(
            self,
            contract: &crate::contract::Contract,
            ctx: SudoCtx,
        ) -> StdResult<Response> {
            contract
                .sudos
                .update(ctx.deps.storage, |count| -> Result<u32, StdError> {
                    Ok(count + 1)
                })?;
            Ok(Response::new())
        }
    }

    #[cw_serde]
    pub enum SudoWrapperMsg {
        CustomSudo(SudoMsg),
        ContractSudo(crate::contract::sv::ContractSudoMsg),
    }

    impl SudoWrapperMsg {
        pub fn dispatch(self, ctx: (DepsMut, Env)) -> StdResult<Response> {
            use SudoWrapperMsg::*;

            match self {
                ContractSudo(msg) => msg.dispatch(&crate::contract::Contract::new(), ctx),
                CustomSudo(msg) => msg.dispatch(&crate::contract::Contract::new(), Into::into(ctx)),
            }
        }
    }
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

    use crate::contract::Contract;

    #[cw_serde]
    pub enum UserExecMsg {
        IncreaseByOne {},
    }

    pub fn increase_by_one(ctx: ExecCtx) -> StdResult<Response> {
        Contract::new()
            .execs
            .update(ctx.deps.storage, |count| -> Result<u32, StdError> {
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

pub mod entry_points {
    use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response, StdError, StdResult};

    use crate::contract::Contract;
    use crate::exec::CustomExecMsg;
    use crate::migrate::MigrateMsg;
    use crate::sudo::SudoWrapperMsg;

    #[entry_point]
    pub fn sudo(deps: DepsMut, env: Env, msg: SudoWrapperMsg) -> StdResult<Response> {
        msg.dispatch((deps, env))
    }

    #[entry_point]
    pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
        Contract::new()
            .migrates
            .update(deps.storage, |count| -> Result<u32, StdError> {
                Ok(count + 1)
            })?;
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
    use cw_storage_plus::Item;
    use sylvia::contract;
    use sylvia::types::{ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx, SudoCtx};

    use crate::CountResponse;

    pub struct Contract {
        pub(crate) execs: Item<'static, u32>,
        pub(crate) sudos: Item<'static, u32>,
        pub(crate) migrates: Item<'static, u32>,
    }

    #[cfg(not(tarpaulin_include))]
    #[contract]
    #[sv::override_entry_point(sudo=crate::entry_points::sudo(crate::sudo::SudoWrapperMsg))]
    #[sv::override_entry_point(migrate=crate::entry_points::migrate(crate::migrate::MigrateMsg))]
    #[sv::override_entry_point(exec=crate::entry_points::execute(crate::exec::CustomExecMsg))]
    #[allow(dead_code)]
    impl Contract {
        #[allow(clippy::new_without_default)]
        #[allow(dead_code)]
        pub fn new() -> Self {
            Self {
                execs: Item::new("execs"),
                sudos: Item::new("sudos"),
                migrates: Item::new("migrates"),
            }
        }

        #[msg(instantiate)]
        pub fn instantiate(&self, ctx: InstantiateCtx) -> StdResult<Response> {
            self.execs.save(ctx.deps.storage, &0)?;
            self.migrates.save(ctx.deps.storage, &0)?;
            self.sudos.save(ctx.deps.storage, &0)?;
            Ok(Response::new())
        }

        #[msg(migrate)]
        pub fn migrate(&self, _ctx: MigrateCtx) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[msg(exec)]
        pub fn increase_by_two(&self, ctx: ExecCtx) -> StdResult<Response> {
            self.execs
                .update(ctx.deps.storage, |count| -> Result<u32, StdError> {
                    Ok(count + 2)
                })?;
            Ok(Response::new())
        }

        #[msg(query)]
        pub fn execs(&self, ctx: QueryCtx) -> StdResult<CountResponse> {
            let count = self.execs.load(ctx.deps.storage)?;
            Ok(CountResponse { count })
        }

        #[msg(query)]
        pub fn sudos(&self, ctx: QueryCtx) -> StdResult<CountResponse> {
            let count = self.sudos.load(ctx.deps.storage)?;
            Ok(CountResponse { count })
        }

        #[msg(query)]
        pub fn migrates(&self, ctx: QueryCtx) -> StdResult<CountResponse> {
            let count = self.migrates.load(ctx.deps.storage)?;
            Ok(CountResponse { count })
        }

        #[msg(sudo)]
        pub fn sudo(&self, _ctx: SudoCtx) -> StdResult<Response> {
            Ok(Response::new())
        }
    }
}

#[cfg(all(test, feature = "mt"))]
mod tests {
    use cosmwasm_std::Addr;
    use cw_multi_test::Executor;
    use sylvia::multitest::App;

    use crate::contract::sv::multitest_utils::CodeId;
    use crate::contract::sv::{ContractExecMsg, ExecMsg};
    use crate::exec::{CustomExecMsg, UserExecMsg};
    use crate::sudo::SudoWrapperMsg;

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

        let msg = SudoWrapperMsg::CustomSudo(crate::sudo::SudoMsg::MoveFunds {
            recipient: "recipient".to_string(),
            amount: vec![],
        });

        contract
            .app
            .app_mut()
            .wasm_sudo(contract.contract_addr.clone(), &msg)
            .unwrap();

        let count = contract.sudos().unwrap().count;
        assert_eq!(count, 1);

        contract.migrate().call(owner, code_id.code_id()).unwrap();
        let count = contract.migrates().unwrap().count;
        assert_eq!(count, 1);

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

        let count = contract.execs().unwrap().count;
        assert_eq!(count, 1);

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

        let count = contract.execs().unwrap().count;
        assert_eq!(count, 3);
    }
}

#![cfg(feature = "mt")]

use cosmwasm_schema::cw_serde;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sylvia::cw_std::{Addr, StdError};
use sylvia::types::Remote;

#[cw_serde]
pub struct ExampleMsg;
impl sylvia::cw_std::CustomMsg for ExampleMsg {}

#[cw_serde]
pub struct ExampleQuery;
impl sylvia::cw_std::CustomQuery for ExampleQuery {}

pub mod counter {
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use sylvia::ctx::{ExecCtx, QueryCtx};
    use sylvia::cw_std::{Response, StdError};
    use sylvia::interface;
    use sylvia::types::{CustomMsg, CustomQuery};

    #[interface]
    pub trait Counter {
        type Error: From<StdError>;
        type ExecC: CustomMsg;
        type QueryC: CustomQuery;
        type CounterT: Serialize + DeserializeOwned + std::fmt::Debug;

        #[sv::msg(exec)]
        fn add(
            &self,
            ctx: ExecCtx<Self::QueryC>,
            value: Self::CounterT,
        ) -> Result<Response<Self::ExecC>, Self::Error>;

        #[sv::msg(query)]
        fn count(&self, ctx: QueryCtx<Self::QueryC>) -> Result<Self::CounterT, Self::Error>;
    }
}

pub mod signed_contract {
    use cw_storage_plus::Item;
    use sylvia::contract;
    use sylvia::ctx::{ExecCtx, InstantiateCtx, QueryCtx};
    use sylvia::cw_std::{Response, StdError, StdResult};

    use crate::counter::Counter;
    use crate::{ExampleMsg, ExampleQuery};

    pub struct SignedContract {
        counter: Item<i32>,
    }

    #[contract]
    #[sv::messages(crate::counter<i32>)]
    #[sv::custom(msg=ExampleMsg, query=ExampleQuery)]
    impl SignedContract {
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self {
                counter: Item::new("counter"),
            }
        }

        #[sv::msg(instantiate)]
        pub fn instantiate(
            &self,
            ctx: InstantiateCtx<ExampleQuery>,
        ) -> StdResult<Response<ExampleMsg>> {
            self.counter.save(ctx.deps.storage, &0)?;
            Ok(Response::new())
        }
    }

    impl Counter for SignedContract {
        type Error = StdError;
        type ExecC = ExampleMsg;
        type QueryC = ExampleQuery;
        type CounterT = i32;

        fn add(
            &self,
            ctx: ExecCtx<Self::QueryC>,
            value: Self::CounterT,
        ) -> Result<Response<Self::ExecC>, Self::Error> {
            self.counter
                .update(ctx.deps.storage, |current_value| -> StdResult<_> {
                    Ok(current_value + value)
                })?;
            Ok(Response::new())
        }

        fn count(&self, ctx: QueryCtx<Self::QueryC>) -> Result<Self::CounterT, Self::Error> {
            let count = self.counter.load(ctx.deps.storage)?;
            Ok(count)
        }
    }
}

pub mod unsigned_contract {
    use cw_storage_plus::Item;
    use sylvia::contract;
    use sylvia::ctx::{ExecCtx, InstantiateCtx, QueryCtx};
    use sylvia::cw_std::{Response, StdError, StdResult};

    use crate::counter::Counter;
    use crate::{ExampleMsg, ExampleQuery};

    pub struct UnsignedContract {
        counter: Item<u32>,
    }

    #[contract]
    #[sv::messages(crate::counter<u32>)]
    #[sv::custom(msg=ExampleMsg, query=ExampleQuery)]
    impl UnsignedContract {
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self {
                counter: Item::new("counter"),
            }
        }

        #[sv::msg(instantiate)]
        pub fn instantiate(
            &self,
            ctx: InstantiateCtx<ExampleQuery>,
        ) -> StdResult<Response<ExampleMsg>> {
            self.counter.save(ctx.deps.storage, &0)?;
            Ok(Response::new())
        }
    }

    impl Counter for UnsignedContract {
        type Error = StdError;
        type ExecC = ExampleMsg;
        type QueryC = ExampleQuery;
        type CounterT = u32;

        fn add(
            &self,
            ctx: ExecCtx<Self::QueryC>,
            value: Self::CounterT,
        ) -> Result<Response<Self::ExecC>, Self::Error> {
            self.counter
                .update(ctx.deps.storage, |current_value| -> StdResult<_> {
                    Ok(current_value + value)
                })?;
            Ok(Response::new())
        }

        fn count(&self, ctx: QueryCtx<Self::QueryC>) -> Result<Self::CounterT, Self::Error> {
            let count = self.counter.load(ctx.deps.storage)?;
            Ok(count)
        }
    }
}

// Making sure `Remote` can be stored in `#[cw_serde]` types.
// This is intentionally a dead code.
// https://github.com/CosmWasm/sylvia/issues/181
#[cw_serde]
#[allow(dead_code)]
pub struct ContractStorage<Contract> {
    remote: Remote<'static, Contract>,
}

#[derive(Serialize, Deserialize)]
pub struct InterfaceStorage<CounterT> {
    interface_remote: Remote<
        'static,
        dyn counter::Counter<
            Error = StdError,
            ExecC = ExampleMsg,
            QueryC = ExampleQuery,
            CounterT = CounterT,
        >,
    >,
}

impl<CounterT> InterfaceStorage<CounterT>
where
    CounterT: Serialize + DeserializeOwned,
{
    pub fn new(contract_addr: Addr) -> Self {
        Self {
            interface_remote: Remote::new(contract_addr),
        }
    }
}

pub mod manager {
    use cw_storage_plus::Item;
    use schemars::JsonSchema;
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use sylvia::ctx::{ExecCtx, InstantiateCtx, QueryCtx};
    use sylvia::cw_std::{Addr, Response, StdError, StdResult};
    use sylvia::{contract, entry_points};

    use crate::counter::sv::{Executor, Querier};
    use crate::{ExampleMsg, ExampleQuery, InterfaceStorage};

    pub struct ManagerContract<CounterT> {
        remote_counter: Item<InterfaceStorage<CounterT>>,
    }

    #[entry_points(generics<i32>)]
    #[contract]
    #[sv::custom(msg=ExampleMsg, query=ExampleQuery)]
    impl<CounterT> ManagerContract<CounterT>
    where
        CounterT: Serialize + DeserializeOwned + std::fmt::Debug + JsonSchema + 'static,
    {
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self {
                remote_counter: Item::new("remote_counter"),
            }
        }

        #[sv::msg(instantiate)]
        pub fn instantiate(
            &self,
            ctx: InstantiateCtx<ExampleQuery>,
            contract_addr: Addr,
        ) -> StdResult<Response<ExampleMsg>> {
            self.remote_counter
                .save(ctx.deps.storage, &InterfaceStorage::new(contract_addr))?;
            Ok(Response::new())
        }

        #[sv::msg(exec)]
        fn add(
            &self,
            ctx: ExecCtx<ExampleQuery>,
            value: CounterT,
        ) -> Result<Response<ExampleMsg>, StdError> {
            let wasm = self
                .remote_counter
                .load(ctx.deps.storage)?
                .interface_remote
                .executor()
                .with_funds(vec![])
                .add(value)?
                .build();
            let resp = Response::new().add_message(wasm);
            Ok(resp)
        }

        #[sv::msg(query)]
        fn count(&self, ctx: QueryCtx<ExampleQuery>) -> Result<CounterT, StdError> {
            let count = self
                .remote_counter
                .load(ctx.deps.storage)?
                .interface_remote
                .querier(&ctx.deps.querier)
                .count()?;

            Ok(count)
        }

        #[sv::msg(exec)]
        fn update_admin(
            &self,
            ctx: ExecCtx<ExampleQuery>,
            new_admin: String,
        ) -> Result<Response<ExampleMsg>, StdError> {
            let wasm = self
                .remote_counter
                .load(ctx.deps.storage)?
                .interface_remote
                .update_admin(&new_admin);
            let resp = Response::new().add_message(wasm);
            Ok(resp)
        }

        #[sv::msg(exec)]
        fn clear_admin(
            &self,
            ctx: ExecCtx<ExampleQuery>,
        ) -> Result<Response<ExampleMsg>, StdError> {
            let wasm = self
                .remote_counter
                .load(ctx.deps.storage)?
                .interface_remote
                .clear_admin();
            let resp = Response::new().add_message(wasm);
            Ok(resp)
        }

        #[sv::msg(query)]
        fn counter_contract(&self, ctx: QueryCtx<ExampleQuery>) -> Result<Addr, StdError> {
            Ok(self
                .remote_counter
                .load(ctx.deps.storage)?
                .interface_remote
                .as_ref()
                .clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{CosmosMsg, WasmMsg};
    use cw_multi_test::{BasicApp, Executor, IntoAddr};
    use sylvia::cw_std::{Addr, StdError};
    use sylvia::multitest::{App, Proxy};
    use sylvia::types::Remote;

    use crate::counter::Counter;
    use crate::manager::sv::mt::{CodeId as ManagerCodeId, ManagerContractProxy};
    use crate::manager::ManagerContract;
    use crate::signed_contract::sv::mt::CodeId as SignedCounterCodeId;
    use crate::signed_contract::SignedContract;
    use crate::unsigned_contract::sv::mt::CodeId as UnsignedCounterCodeId;
    use crate::unsigned_contract::UnsignedContract;
    use crate::{ExampleMsg, ExampleQuery};

    type ExampleApp = BasicApp<ExampleMsg, ExampleQuery>;

    #[test]
    fn remote_generation() {
        // interface
        let addr = Addr::unchecked("counter_interface");
        let _ = Remote::<
            'static,
            dyn Counter<
                Error = StdError,
                ExecC = ExampleMsg,
                QueryC = ExampleQuery,
                CounterT = i32,
            >,
        >::new(addr.clone());
        let borrowed_remote = Remote::<()>::borrowed(&addr);
        assert_eq!(&addr, borrowed_remote.as_ref());

        // contract
        let addr = Addr::unchecked("counter_contract");
        let _ = Remote::<SignedContract>::new(addr.clone());
        let borrowed_remote = Remote::<UnsignedContract>::borrowed(&addr);
        assert_eq!(&addr, borrowed_remote.as_ref());
    }

    fn setup<'a>(
        app: &'a App<ExampleApp>,
        owner: &'a Addr,
    ) -> (
        Proxy<'a, ExampleApp, ManagerContract<i32>>,
        Proxy<'a, ExampleApp, ManagerContract<u32>>,
    ) {
        // Manager operating on signed numbers
        let signed_counter_code_id = SignedCounterCodeId::store_code(app);

        let signed_counter_contract = signed_counter_code_id
            .instantiate()
            .with_label("Signed counter contract")
            .call(owner)
            .unwrap();

        let manager_code_id = ManagerCodeId::store_code(app);

        let signed_manager_contract = manager_code_id
            .instantiate(signed_counter_contract.contract_addr.clone())
            .with_label("Manager contract")
            .call(owner)
            .unwrap();

        // Manager operating on unsigned numbers
        let unsigned_counter_code_id = UnsignedCounterCodeId::store_code(app);

        let unsigned_counter_contract = unsigned_counter_code_id
            .instantiate()
            .with_label("Unsigned counter contract")
            .with_admin(Some(owner.as_str()))
            .call(owner)
            .unwrap();

        let manager_code_id = ManagerCodeId::store_code(app);

        let unsigned_manager_contract = manager_code_id
            .instantiate(unsigned_counter_contract.contract_addr.clone())
            .with_label("Manager contract")
            .call(owner)
            .unwrap();

        // Set manager contract as an admin of the counter contract
        app.app_mut()
            .execute(
                owner.clone(),
                CosmosMsg::Wasm(WasmMsg::UpdateAdmin {
                    contract_addr: unsigned_counter_contract.contract_addr.to_string(),
                    admin: unsigned_manager_contract.contract_addr.to_string(),
                }),
            )
            .unwrap();

        (signed_manager_contract, unsigned_manager_contract)
    }

    #[test]
    fn call_remote() {
        let owner = "owner".into_addr();
        let app = App::<cw_multi_test::BasicApp<ExampleMsg, ExampleQuery>>::custom(|_, _, _| {});
        let (signed_manager_contract, unsigned_manager_contract) = setup(&app, &owner);

        assert_eq!(signed_manager_contract.count().unwrap(), 0);

        signed_manager_contract.add(5).call(&owner).unwrap();
        assert_eq!(signed_manager_contract.count().unwrap(), 5);

        assert_eq!(unsigned_manager_contract.count().unwrap(), 0);

        unsigned_manager_contract.add(5).call(&owner).unwrap();
        assert_eq!(unsigned_manager_contract.count().unwrap(), 5);
    }

    #[test]
    fn update_admin() {
        let owner = "owner".into_addr();
        let app = App::<cw_multi_test::BasicApp<ExampleMsg, ExampleQuery>>::custom(|_, _, _| {});
        let (_, unsigned_manager_contract) = setup(&app, &owner);
        let new_admin = "new_admin".into_addr();

        let unsigned_counter_contract_addr = unsigned_manager_contract.counter_contract().unwrap();

        // Initial admin should be the manager_contract
        let contract_info = app
            .querier()
            .query_wasm_contract_info(unsigned_counter_contract_addr.clone())
            .unwrap();
        assert_eq!(
            contract_info.admin,
            Some(unsigned_manager_contract.contract_addr.clone())
        );

        // Add new admin
        unsigned_manager_contract
            .update_admin(new_admin.to_string())
            .call(&owner)
            .unwrap();

        let contract_info = app
            .querier()
            .query_wasm_contract_info(unsigned_counter_contract_addr)
            .unwrap();
        assert_eq!(contract_info.admin, Some(new_admin.clone()));
    }

    #[test]
    fn clear_admin() {
        let owner = "owner".into_addr();
        let app = App::<cw_multi_test::BasicApp<ExampleMsg, ExampleQuery>>::custom(|_, _, _| {});
        let (_, unsigned_manager_contract) = setup(&app, &owner);

        let unsigned_counter_contract_addr = unsigned_manager_contract.counter_contract().unwrap();

        // Initial admin should be the manager_contract
        let contract_info = app
            .querier()
            .query_wasm_contract_info(unsigned_counter_contract_addr.clone())
            .unwrap();
        assert_eq!(
            contract_info.admin,
            Some(unsigned_manager_contract.contract_addr.clone())
        );

        // Clear admin
        unsigned_manager_contract
            .clear_admin()
            .call(&owner)
            .unwrap();

        let contract_info = app
            .querier()
            .query_wasm_contract_info(unsigned_counter_contract_addr.clone())
            .unwrap();
        assert_eq!(contract_info.admin, None);
    }
}

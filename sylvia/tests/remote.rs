use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sylvia::types::Remote;

#[cw_serde]
pub struct ExampleMsg;
impl cosmwasm_std::CustomMsg for ExampleMsg {}

#[cw_serde]
pub struct ExampleQuery;
impl cosmwasm_std::CustomQuery for ExampleQuery {}

pub mod counter {
    use cosmwasm_std::{Response, StdError};
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use sylvia::interface;
    use sylvia::types::{CustomMsg, CustomQuery, ExecCtx, QueryCtx};

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
    use cosmwasm_std::{Response, StdError, StdResult};
    use cw_storage_plus::Item;
    use sylvia::contract;
    use sylvia::types::{ExecCtx, InstantiateCtx, QueryCtx};

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
    use cosmwasm_std::{Response, StdError, StdResult};
    use cw_storage_plus::Item;
    use sylvia::contract;
    use sylvia::types::{ExecCtx, InstantiateCtx, QueryCtx};

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

// Making sure `Remote` can be stored in `#[cw_serde]` types
#[cw_serde]
#[allow(dead_code)]
pub struct ContractStorage<'a, Contract> {
    remote: Remote<'a, Contract>,
}

#[derive(Serialize, Deserialize)]
pub struct InterfaceStorage<'a, CounterT> {
    interface_remote: Remote<
        'a,
        dyn counter::Counter<
            Error = StdError,
            ExecC = ExampleMsg,
            QueryC = ExampleQuery,
            CounterT = CounterT,
        >,
    >,
}

impl<CounterT> InterfaceStorage<'_, CounterT>
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
    use cosmwasm_std::{Addr, Response, StdError, StdResult};
    use cw_storage_plus::Item;
    use schemars::JsonSchema;
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use sylvia::contract;
    use sylvia::types::{ExecCtx, InstantiateCtx, QueryCtx};

    use crate::counter::sv::{Executor, Querier};
    use crate::{ExampleMsg, ExampleQuery, InterfaceStorage};

    pub struct ManagerContract<'a, CounterT> {
        remote_counter: Item<InterfaceStorage<'a, CounterT>>,
    }

    #[contract]
    // Due to how `cw_multi_test::App` is constructed to test two contracts they have
    // to use the same custom types.
    // Not sure if this limits testing of some real life scenarios.
    #[sv::custom(msg=ExampleMsg, query=ExampleQuery)]
    impl<'a, CounterT> ManagerContract<'a, CounterT>
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
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, StdError};
    use cw_multi_test::{BasicApp, IntoBech32};
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
    const OWNER: &str = "owner";

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

    fn setup(
        app: &App<ExampleApp>,
    ) -> (
        Proxy<ExampleApp, ManagerContract<i32>>,
        Proxy<ExampleApp, ManagerContract<u32>>,
    ) {
        // Manager operating on signed numbers
        let signed_counter_code_id = SignedCounterCodeId::store_code(app);

        let signed_counter_contract = signed_counter_code_id
            .instantiate()
            .with_label("Signed counter contract")
            .call(&OWNER.into_bech32())
            .unwrap();

        let manager_code_id = ManagerCodeId::store_code(app);

        let signed_manager_contract = manager_code_id
            .instantiate(signed_counter_contract.contract_addr.clone())
            .with_label("Manager contract")
            .call(&OWNER.into_bech32())
            .unwrap();

        // Manager operating on unsigned numbers
        let unsigned_counter_code_id = UnsignedCounterCodeId::store_code(app);

        let unsigned_counter_contract = unsigned_counter_code_id
            .instantiate()
            .with_label("Unsigned counter contract")
            .call(&OWNER.into_bech32())
            .unwrap();

        let manager_code_id = ManagerCodeId::store_code(app);

        let unsigned_manager_contract = manager_code_id
            .instantiate(unsigned_counter_contract.contract_addr.clone())
            .with_label("Manager contract")
            .call(&OWNER.into_bech32())
            .unwrap();

        (signed_manager_contract, unsigned_manager_contract)
    }

    #[test]
    fn call_remote() {
        let app = App::<cw_multi_test::BasicApp<ExampleMsg, ExampleQuery>>::custom(|_, _, _| {});
        let (signed_manager_contract, unsigned_manager_contract) = setup(&app);

        assert_eq!(signed_manager_contract.count().unwrap(), 0);

        signed_manager_contract
            .add(5)
            .call(&OWNER.into_bech32())
            .unwrap();
        assert_eq!(signed_manager_contract.count().unwrap(), 5);

        assert_eq!(unsigned_manager_contract.count().unwrap(), 0);

        unsigned_manager_contract
            .add(5)
            .call(&OWNER.into_bech32())
            .unwrap();
        assert_eq!(unsigned_manager_contract.count().unwrap(), 5);
    }
}

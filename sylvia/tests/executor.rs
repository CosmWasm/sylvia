#![cfg(feature = "mt")]

use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sylvia::ctx::InstantiateCtx;
use sylvia::cw_std::{Addr, Response, StdResult};
use sylvia::{contract, entry_points};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug, Default)]
pub struct CountResponse {
    pub count: u64,
}

pub mod counter {
    use sylvia::ctx::{ExecCtx, QueryCtx};
    use sylvia::cw_std::{Response, StdError, StdResult};
    use sylvia::interface;

    use crate::CountResponse;

    #[interface]
    #[sv::custom(msg=sylvia::cw_std::Empty, query=sylvia::cw_std::Empty)]
    pub trait Counter {
        type Error: From<StdError>;

        #[sv::msg(query)]
        fn count(&self, ctx: QueryCtx) -> StdResult<CountResponse>;

        #[sv::msg(exec)]
        fn decrease(&self, ctx: ExecCtx) -> StdResult<Response>;
    }
}

use counter::sv::Executor as InterfaceExecutor;
use counter::Counter;
use sv::Executor as ContractExecutor;
use sylvia::ctx::{ExecCtx, QueryCtx};
use sylvia::cw_std::StdError;

impl Counter for CounterContract {
    type Error = StdError;

    fn count(&self, ctx: QueryCtx) -> StdResult<CountResponse> {
        let count = self.count.load(ctx.deps.storage)?;
        Ok(CountResponse { count })
    }

    fn decrease(&self, ctx: ExecCtx) -> StdResult<Response> {
        self.count.update(ctx.deps.storage, |count| {
            let count = count.saturating_sub(1);
            Ok::<_, StdError>(count)
        })?;

        Ok(Response::new())
    }
}

pub struct CounterContract {
    pub count: Item<u64>,
    pub remote_contract: Item<sylvia::types::Remote<'static, CounterContract>>,
    pub remote_interface: Item<sylvia::types::Remote<'static, dyn Counter<Error = StdError>>>,
}

#[entry_points]
#[contract]
#[sv::messages(counter)]
impl CounterContract {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            count: Item::new("count"),
            remote_contract: Item::new("remote_contract"),
            remote_interface: Item::new("remote_interface"),
        }
    }

    #[sv::msg(instantiate)]
    fn instantiate(&self, ctx: InstantiateCtx, remote_addr: Addr) -> StdResult<Response> {
        self.count.save(ctx.deps.storage, &100)?;
        self.remote_contract.save(
            ctx.deps.storage,
            &sylvia::types::Remote::new(remote_addr.clone()),
        )?;
        self.remote_interface
            .save(ctx.deps.storage, &sylvia::types::Remote::new(remote_addr))?;
        Ok(Response::new())
    }

    #[sv::msg(exec)]
    fn increase(&self, ctx: ExecCtx) -> StdResult<Response> {
        self.count.update(ctx.deps.storage, |count| {
            let count = count.saturating_add(1);
            Ok::<_, StdError>(count)
        })?;

        Ok(Response::new())
    }

    #[sv::msg(exec)]
    fn increase_in_other_contract(&self, ctx: ExecCtx) -> StdResult<Response> {
        let remote = self.remote_contract.load(ctx.deps.storage)?;
        let increase_msg = remote.executor().increase()?.build();
        Ok(Response::new().add_message(increase_msg))
    }

    #[sv::msg(exec)]
    fn decrease_in_other_contract(&self, ctx: ExecCtx) -> StdResult<Response> {
        let remote = self.remote_interface.load(ctx.deps.storage)?;
        let decrease_msg = remote.executor().decrease()?.build();
        Ok(Response::new().add_message(decrease_msg))
    }
}

#[cfg(test)]
mod tests {
    use cw_multi_test::IntoBech32;
    use sylvia::cw_std::Addr;
    use sylvia::multitest::App;

    use crate::counter::sv::mt::CounterProxy;
    use crate::sv::mt::{CodeId, CounterContractProxy};

    #[test]
    fn call_querier() {
        let app = App::default();
        let code_id = CodeId::store_code(&app);

        let owner = "owner".into_bech32();

        let first_contract = code_id
            .instantiate(Addr::unchecked("remote"))
            .with_label("First Counter")
            .call(&owner)
            .unwrap();

        let second_contract = code_id
            .instantiate(first_contract.contract_addr.clone())
            .with_label("Second Counter")
            .call(&owner)
            .unwrap();

        second_contract
            .increase_in_other_contract()
            .call(&owner)
            .unwrap();

        let resp = first_contract.count().unwrap();
        assert_eq!(resp.count, 101);

        second_contract
            .decrease_in_other_contract()
            .call(&owner)
            .unwrap();
        second_contract
            .decrease_in_other_contract()
            .call(&owner)
            .unwrap();

        let resp = first_contract.count().unwrap();
        assert_eq!(resp.count, 99);
    }
}

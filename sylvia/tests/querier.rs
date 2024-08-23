#![cfg(feature = "mt")]

use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sylvia::cw_std::{Addr, Response, StdResult};
use sylvia::types::InstantiateCtx;
use sylvia::{contract, entry_points};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug, Default)]
pub struct CountResponse {
    pub count: u64,
}

pub mod counter {
    use sylvia::cw_std::{Response, StdError, StdResult};
    use sylvia::interface;
    use sylvia::types::{ExecCtx, QueryCtx};

    use crate::CountResponse;

    #[interface]
    #[sv::custom(msg=sylvia::cw_std::Empty, query=sylvia::cw_std::Empty)]
    pub trait Counter {
        type Error: From<StdError>;

        #[sv::msg(query)]
        fn count(&self, ctx: QueryCtx) -> StdResult<CountResponse>;

        #[sv::msg(exec)]
        fn copy_count(&self, ctx: ExecCtx) -> StdResult<Response>;

        #[sv::msg(exec)]
        fn set_count(&self, ctx: ExecCtx, new_count: u64) -> StdResult<Response>;

        #[sv::msg(exec)]
        fn decrease_by_count(&self, ctx: ExecCtx) -> StdResult<Response>;
    }
}

pub mod impl_counter {
    use crate::counter::sv::Querier;
    use crate::counter::Counter;
    use crate::CountResponse;
    use sylvia::cw_std::{Response, StdError, StdResult};
    use sylvia::types::{ExecCtx, QueryCtx};

    impl Counter for super::CounterContract {
        type Error = StdError;

        fn count(&self, ctx: QueryCtx) -> StdResult<CountResponse> {
            let count = self.count.load(ctx.deps.storage)?;
            Ok(CountResponse { count })
        }

        fn set_count(&self, ctx: ExecCtx, new_count: u64) -> StdResult<Response> {
            self.count.save(ctx.deps.storage, &new_count)?;
            Ok(Response::new())
        }

        fn copy_count(&self, ctx: ExecCtx) -> StdResult<Response> {
            let other_count = self
                .remote
                .load(ctx.deps.storage)?
                .querier(&ctx.deps.querier)
                .count()?
                .count;

            self.count.save(ctx.deps.storage, &other_count)?;
            Ok(Response::new())
        }

        fn decrease_by_count(&self, ctx: ExecCtx) -> StdResult<Response> {
            let remote = self.remote.load(ctx.deps.storage)?;
            let other_count = sylvia::types::BoundQuerier::<_, Self>::borrowed(
                remote.as_ref(),
                &ctx.deps.querier,
            )
            .count()?
            .count;
            self.count.update(ctx.deps.storage, |count| {
                let count = count.saturating_sub(other_count);
                Ok::<_, StdError>(count)
            })?;

            Ok(Response::new())
        }
    }
}

pub struct CounterContract {
    pub count: Item<u64>,
    pub remote: Item<sylvia::types::Remote<'static, CounterContract>>,
}

#[entry_points]
#[contract]
#[sv::messages(counter)]
impl CounterContract {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            count: Item::new("count"),
            remote: Item::new("remote"),
        }
    }

    #[sv::msg(instantiate)]
    fn instantiate(&self, ctx: InstantiateCtx, remote_addr: Addr) -> StdResult<Response> {
        self.count.save(ctx.deps.storage, &0)?;
        self.remote
            .save(ctx.deps.storage, &sylvia::types::Remote::new(remote_addr))?;
        Ok(Response::new())
    }
}

#[cfg(test)]
mod tests {
    use cw_multi_test::IntoBech32;
    use sylvia::cw_std::testing::mock_dependencies;
    use sylvia::cw_std::{Addr, Empty, QuerierWrapper};
    use sylvia::multitest::App;

    use crate::counter::sv::mt::CounterProxy;
    use crate::sv::mt::CodeId;

    #[test]
    fn querier_generation() {
        let deps = mock_dependencies();
        let querier_wrapper = QuerierWrapper::<Empty>::new(&deps.querier);
        let remote_addr = Addr::unchecked("remote");

        // Remote generation
        let remote = sylvia::types::Remote::<super::CounterContract>::new(remote_addr.clone());
        let _: sylvia::types::BoundQuerier<_, _> = remote.querier(&querier_wrapper);
        let remote = sylvia::types::Remote::<super::CounterContract>::new(remote_addr.clone());
        let _: sylvia::types::BoundQuerier<_, _> = remote.querier(&querier_wrapper);

        // Querier generation
        let _ = sylvia::types::BoundQuerier::<_, super::CounterContract>::borrowed(
            &remote_addr,
            &querier_wrapper,
        );
        let querier = sylvia::types::BoundQuerier::borrowed(&remote_addr, &querier_wrapper);

        let _ = sylvia::types::BoundQuerier::<_, super::CounterContract>::from(&querier);
    }

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

        first_contract.set_count(42).call(&owner).unwrap();

        let resp = second_contract.count().unwrap();
        assert_eq!(resp.count, 0);

        second_contract.copy_count().call(&owner).unwrap();

        let resp = second_contract.count().unwrap();
        assert_eq!(resp.count, 42);

        second_contract.decrease_by_count().call(&owner).unwrap();

        let resp = second_contract.count().unwrap();
        assert_eq!(resp.count, 0);
    }
}

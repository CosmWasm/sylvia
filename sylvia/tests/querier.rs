#![cfg(feature = "mt")]

use cosmwasm_std::{Addr, Response, StdResult};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sylvia::contract;
use sylvia::types::InstantiateCtx;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug, Default)]
pub struct CountResponse {
    pub count: u64,
}

pub mod counter {
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::interface;
    use sylvia::types::{ExecCtx, QueryCtx};

    use crate::CountResponse;

    #[interface]
    pub trait Counter {
        type Error: From<StdError>;

        #[msg(query)]
        fn count(&self, ctx: QueryCtx) -> StdResult<CountResponse>;

        #[msg(exec)]
        fn copy_count(&self, ctx: ExecCtx) -> StdResult<Response>;

        #[msg(exec)]
        fn set_count(&self, ctx: ExecCtx, new_count: u64) -> StdResult<Response>;

        #[msg(exec)]
        fn decrease_by_count(&self, ctx: ExecCtx) -> StdResult<Response>;
    }
}

pub mod impl_counter {
    use crate::counter::sv::Querier;
    use crate::counter::Counter;
    use crate::CountResponse;
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::contract;
    use sylvia::types::{ExecCtx, QueryCtx};

    #[contract(module=crate)]
    #[messages(crate::counter)]
    impl Counter for super::CounterContract<'_> {
        type Error = StdError;

        #[msg(query)]
        fn count(&self, ctx: QueryCtx) -> StdResult<CountResponse> {
            let count = self.count.load(ctx.deps.storage)?;
            Ok(CountResponse { count })
        }

        #[msg(exec)]
        fn set_count(&self, ctx: ExecCtx, new_count: u64) -> StdResult<Response> {
            self.count.save(ctx.deps.storage, &new_count)?;
            Ok(Response::new())
        }

        #[msg(exec)]
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

        #[msg(exec)]
        fn decrease_by_count(&self, ctx: ExecCtx) -> StdResult<Response> {
            let remote = self.remote.load(ctx.deps.storage)?;
            let other_count =
                crate::counter::sv::BoundQuerier::borrowed(remote.as_ref(), &ctx.deps.querier)
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

pub struct CounterContract<'a> {
    pub count: Item<'static, u64>,
    pub remote: Item<'static, sv::Remote<'a>>,
}

#[contract]
#[messages(counter)]
impl CounterContract<'_> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            count: Item::new("count"),
            remote: Item::new("remote"),
        }
    }

    #[msg(instantiate)]
    fn instantiate(&self, ctx: InstantiateCtx, remote_addr: Addr) -> StdResult<Response> {
        self.count.save(ctx.deps.storage, &0)?;
        self.remote
            .save(ctx.deps.storage, &sv::Remote::new(remote_addr))?;
        Ok(Response::new())
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{Addr, Empty, QuerierWrapper};
    use sylvia::multitest::App;

    use crate::impl_counter::sv::test_utils::Counter;
    use crate::sv::multitest_utils::CodeId;

    #[test]
    fn querier_generation() {
        let deps = mock_dependencies();
        let querier_wrapper = QuerierWrapper::<Empty>::new(&deps.querier);
        let remote_addr = Addr::unchecked("remote");

        // Remote generation
        let remote = super::counter::sv::Remote::new(remote_addr.clone());
        let _: super::counter::sv::BoundQuerier<_> = remote.querier(&querier_wrapper);
        let remote = super::sv::Remote::new(remote_addr.clone());
        let _: super::sv::BoundQuerier<_> = remote.querier(&querier_wrapper);

        // Querier generation
        let _ = super::counter::sv::BoundQuerier::borrowed(&remote_addr, &querier_wrapper);
        let querier = super::sv::BoundQuerier::borrowed(&remote_addr, &querier_wrapper);

        let _ = super::counter::sv::BoundQuerier::from(&querier);
    }

    #[test]
    fn call_querier() {
        let app = App::default();
        let code_id = CodeId::store_code(&app);

        let owner = "owner";

        let first_contract = code_id
            .instantiate(Addr::unchecked("remote"))
            .with_label("First Counter")
            .call(owner)
            .unwrap();

        let second_contract = code_id
            .instantiate(first_contract.contract_addr.clone())
            .with_label("Second Counter")
            .call(owner)
            .unwrap();

        first_contract.set_count(42).call(owner).unwrap();

        let resp = second_contract.count().unwrap();
        assert_eq!(resp.count, 0);

        second_contract.copy_count().call(owner).unwrap();

        let resp = second_contract.count().unwrap();
        assert_eq!(resp.count, 42);

        second_contract.decrease_by_count().call(owner).unwrap();

        let resp = second_contract.count().unwrap();
        assert_eq!(resp.count, 0);
    }
}

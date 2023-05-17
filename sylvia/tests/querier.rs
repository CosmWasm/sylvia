use cosmwasm_std::{Addr, Response, StdError, StdResult};
use cw_storage_plus::Item;
use serde::{Deserialize, Serialize};
use sylvia::types::{ExecCtx, InstantiateCtx, QueryCtx};
use sylvia::{contract, interface};

#[derive(
    Serialize, Deserialize, Clone, PartialEq, Eq, sylvia::schemars::JsonSchema, Debug, Default,
)]
pub struct CountResponse {
    pub count: u64,
}

pub mod counter {
    use super::*;

    #[interface]
    pub trait Counter {
        type Error: From<StdError>;

        #[msg(query)]
        fn count(&self, ctx: QueryCtx) -> StdResult<CountResponse>;

        #[msg(exec)]
        fn copy_count(&self, ctx: ExecCtx, contract_addr: Addr) -> StdResult<Response>;

        #[msg(exec)]
        fn set_count(&self, ctx: ExecCtx, new_count: u64) -> StdResult<Response>;
    }

    #[contract]
    #[messages(counter as Counter)]
    impl Counter for CounterContract {
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
        fn copy_count(&self, ctx: ExecCtx, contract_addr: Addr) -> StdResult<Response> {
            let other_count = counter::Remote::new(contract_addr)
                .querier(&ctx.deps.querier)
                .count()?
                .count;
            self.count.save(ctx.deps.storage, &other_count)?;
            Ok(Response::new())
        }
    }
}

pub struct CounterContract {
    pub(crate) count: Item<'static, u64>,
}

#[contract]
#[messages(counter as Counter)]
impl CounterContract {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            count: Item::new("count"),
        }
    }

    #[msg(instantiate)]
    fn instantiate(&self, ctx: InstantiateCtx) -> StdResult<Response> {
        self.count.save(ctx.deps.storage, &0)?;
        Ok(Response::new())
    }
}

#[cfg(test)]
mod tests {
    use sylvia::multitest::App;

    use crate::counter::test_utils::Counter;
    use crate::multitest_utils::CodeId;

    #[test]
    fn call_querier() {
        let app = App::default();
        let code_id = CodeId::store_code(&app);

        let owner = "owner";

        let first_contract = code_id
            .instantiate()
            .with_label("First Counter")
            .call(owner)
            .unwrap();

        let second_contract = code_id
            .instantiate()
            .with_label("Second Counter")
            .call(owner)
            .unwrap();

        first_contract
            .counter_proxy()
            .set_count(42)
            .call(owner)
            .unwrap();

        let resp = second_contract.counter_proxy().count().unwrap();
        assert_eq!(resp.count, 0);

        second_contract
            .counter_proxy()
            .copy_count(first_contract.contract_addr)
            .call(owner)
            .unwrap();

        let resp = second_contract.counter_proxy().count().unwrap();
        assert_eq!(resp.count, 42);
    }
}

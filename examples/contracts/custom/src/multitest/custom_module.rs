use cw_storage_plus::Item;
use std::fmt::Debug;
use sylvia::anyhow::Result as AnyResult;
use sylvia::cw_multi_test::{AppResponse, CosmosRouter, Module};
use sylvia::cw_std::{
    to_json_binary, Addr, Api, Binary, BlockInfo, CustomQuery, Empty, Querier, StdError, StdResult,
    Storage,
};
use sylvia::serde::de::DeserializeOwned;

use crate::messages::{CountResponse, CounterMsg, CounterQuery};

pub struct CustomModule {
    pub counter: Item<u64>,
}

impl Default for CustomModule {
    fn default() -> Self {
        Self {
            counter: Item::new("counter"),
        }
    }
}

impl CustomModule {
    pub fn save_counter(&self, storage: &mut dyn Storage, value: u64) -> StdResult<()> {
        self.counter.save(storage, &value)
    }
}

impl Module for CustomModule {
    type ExecT = CounterMsg;
    type QueryT = CounterQuery;
    type SudoT = Empty;

    fn execute<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        _sender: Addr,
        msg: Self::ExecT,
    ) -> AnyResult<AppResponse>
    where
        ExecC: Debug + Clone + PartialEq + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        match msg {
            CounterMsg::Increment {} => {
                self.counter
                    .update(storage, |value| Ok::<_, StdError>(value + 1))?;
                Ok(AppResponse::default())
            }
        }
    }

    fn sudo<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        _storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        _msg: Self::SudoT,
    ) -> AnyResult<AppResponse>
    where
        ExecC: Debug + Clone + PartialEq + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        Ok(AppResponse::default())
    }

    fn query(
        &self,
        _api: &dyn Api,
        storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        request: Self::QueryT,
    ) -> AnyResult<Binary> {
        match request {
            CounterQuery::Count {} => {
                let count = self.counter.load(storage)?;
                let res = CountResponse { count };
                to_json_binary(&res).map_err(Into::into)
            }
        }
    }
}

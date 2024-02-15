use cosmwasm_schema::schemars::JsonSchema;
use cosmwasm_std::{
    to_json_binary, Addr, Api, Binary, BlockInfo, CustomQuery, Querier, StdError, StdResult,
    Storage,
};
use cw_multi_test::{AppResponse, CosmosRouter, Module};
use cw_storage_plus::Item;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::{
    contract::sv::{ContractExecMsg, ContractSudoMsg},
    messages::{CountResponse, CounterMsg, CounterQuery, CounterSudo},
};

pub struct CustomModule {
    pub exec_counter: Item<'static, u64>,
    pub sudo_counter: Item<'static, u64>,
}

impl Default for CustomModule {
    fn default() -> Self {
        Self {
            exec_counter: Item::new("exec_counter"),
            sudo_counter: Item::new("sudo_counter"),
        }
    }
}

impl CustomModule {
    pub fn init_counter(&self, storage: &mut dyn Storage) -> StdResult<()> {
        self.exec_counter.save(storage, &0)?;
        self.sudo_counter.save(storage, &0)
    }
}

impl Module for CustomModule {
    type ExecT = ContractExecMsg;
    type QueryT = ContractQueryMsg;
    type SudoT = ContractSudoMsg;

    fn execute<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        _sender: Addr,
        msg: Self::ExecT,
    ) -> anyhow::Result<AppResponse>
    where
        ExecC: Debug + Clone + PartialEq + JsonSchema + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        match msg {
            CounterMsg::Increment {} => {
                self.exec_counter
                    .update(storage, |value| Ok::<_, StdError>(value + 1))?;
                Ok(AppResponse::default())
            }
        }
    }

    fn sudo<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        msg: Self::SudoT,
    ) -> anyhow::Result<AppResponse>
    where
        ExecC: Debug + Clone + PartialEq + JsonSchema + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        match msg {
            CounterSudo::Increment {} => {
                self.sudo_counter
                    .update(storage, |value| Ok::<_, StdError>(value + 1))?;
                Ok(AppResponse::default())
            }
        }
    }

    fn query(
        &self,
        _api: &dyn Api,
        storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        request: Self::QueryT,
    ) -> anyhow::Result<Binary> {
        match request {
            CounterQuery::Exec {} => {
                let count = self.exec_counter.load(storage)?;
                let res = CountResponse { count };
                to_json_binary(&res).map_err(Into::into)
            }
            CounterQuery::Sudo {} => {
                let count = self.sudo_counter.load(storage)?;
                let res = CountResponse { count };
                to_json_binary(&res).map_err(Into::into)
            }
        }
    }
}

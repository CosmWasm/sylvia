use cosmwasm_std::{Addr, Coin, CosmosMsg, StdResult};
use cw_multi_test::{App, Executor};
use cw_utils::Expiration;

use crate::contract::multitest_utils;
use crate::contract::{Cw1SubkeysContract, ExecMsg, InstantiateMsg, QueryMsg};
use crate::error::ContractError;
use crate::responses::{AllAllowancesResponse, AllPermissionsResponse};
use crate::state::{Allowance, Permissions};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cw1SubkeysCodeId(u64);

impl Cw1SubkeysCodeId {
    pub fn store_code(app: &mut App) -> Self {
        let code_id = app.store_code(Box::new(Cw1SubkeysContract::new()));
        Self(code_id)
    }

    #[track_caller]
    pub fn instantiate(
        self,
        app: &mut App,
        sender: &Addr,
        admins: &[&Addr],
        mutable: bool,
        label: &str,
    ) -> Result<Cw1SubkeysProxy, ContractError> {
        let admins = admins.iter().map(|a| a.to_string()).collect();
        let msg = InstantiateMsg { admins, mutable };

        app.instantiate_contract(self.0, sender.clone(), &msg, &[], label, None)
            .map_err(|err| err.downcast().unwrap())
            .map(Cw1SubkeysProxy)
    }
}

pub struct Cw1SubkeysProxy(Addr);

impl Cw1SubkeysProxy {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    #[track_caller]
    pub fn increase_allowance(
        &self,
        app: &mut App,
        sender: &Addr,
        spender: &Addr,
        amount: Coin,
        expires: impl Into<Option<Expiration>>,
    ) -> Result<(), ContractError> {
        let msg = ExecMsg::IncreaseAllowance {
            spender: spender.to_string(),
            amount,
            expires: expires.into(),
        };

        app.execute_contract(sender.clone(), self.0.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn set_permission(
        &self,
        app: &mut App,
        sender: &Addr,
        spender: &Addr,
        permissions: Permissions,
    ) -> Result<(), ContractError> {
        let msg = ExecMsg::SetPermissions {
            spender: spender.to_string(),
            permissions,
        };

        app.execute_contract(sender.clone(), self.0.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    pub fn allowance(&self, app: &App, spender: &Addr) -> StdResult<Allowance> {
        let msg = QueryMsg::Allowance {
            spender: spender.to_string(),
        };

        app.wrap().query_wasm_smart(self.0.clone(), &msg)
    }

    pub fn all_allowances<'a>(
        &self,
        app: &App,
        start_after: impl Into<Option<&'a Addr>>,
        limit: impl Into<Option<u32>>,
    ) -> StdResult<AllAllowancesResponse> {
        let msg = QueryMsg::AllAllowances {
            start_after: start_after.into().map(|a| a.to_string()),
            limit: limit.into(),
        };

        app.wrap().query_wasm_smart(self.0.clone(), &msg)
    }

    pub fn permissions(&self, app: &App, spender: &Addr) -> StdResult<Permissions> {
        let msg = QueryMsg::Permissions {
            spender: spender.to_string(),
        };

        app.wrap().query_wasm_smart(self.0.clone(), &msg)
    }

    pub fn all_permissions<'a>(
        &self,
        app: &App,
        start_after: impl Into<Option<&'a Addr>>,
        limit: impl Into<Option<u32>>,
    ) -> StdResult<AllPermissionsResponse> {
        let msg = QueryMsg::AllPermissions {
            start_after: start_after.into().map(|a| a.to_string()),
            limit: limit.into(),
        };

        app.wrap().query_wasm_smart(self.0.clone(), &msg)
    }

    pub fn can_execute(
        &self,
        app: &App,
        sender: String,
        msg: CosmosMsg,
    ) -> StdResult<cw1::CanExecuteResp> {
        let msg = cw1::QueryMsg::CanExecute { sender, msg };

        app.wrap().query_wasm_smart(self.0.clone(), &msg)
    }

    #[track_caller]
    pub fn execute(
        &self,
        app: &mut App,
        sender: Addr,
        msgs: Vec<CosmosMsg>,
        send_funds: &[Coin],
    ) -> Result<(), ContractError> {
        let msg = cw1::ExecMsg::Execute { msgs };

        app.execute_contract(sender, self.0.clone(), &msg, send_funds)
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }
}

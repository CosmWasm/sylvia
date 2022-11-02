use cosmwasm_std::{Addr, Binary, StdResult, Uint128};
use cw20::{
    AllAllowancesResponse, AllSpenderAllowancesResponse, AllowanceResponse, BalanceResponse,
};

use cw_multi_test::{App, Executor};
use cw_utils::Expiration;

use crate::contract::{Cw20Base, InstantiateMsg, InstantiateMsgData, QueryMsg};
use crate::error::ContractError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cw20BaseCodeId(u64);

impl Cw20BaseCodeId {
    pub fn store_code(app: &mut App) -> Self {
        let code_id = app.store_code(Box::new(Cw20Base::new()));
        Self(code_id)
    }

    #[track_caller]
    pub fn instantiate(
        self,
        app: &mut App,
        sender: &Addr,
        data: InstantiateMsgData,
        label: &str,
    ) -> Result<Cw20BaseProxy, ContractError> {
        let msg = InstantiateMsg { data };

        app.instantiate_contract(self.0, sender.clone(), &msg, &[], label, None)
            .map_err(|err| err.downcast().unwrap())
            .map(Cw20BaseProxy)
    }
}

pub struct Cw20BaseProxy(Addr);

impl Cw20BaseProxy {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    pub fn balance(&self, app: &App, address: String) -> StdResult<BalanceResponse> {
        let msg = QueryMsg::Balance { address };

        app.wrap().query_wasm_smart(self.0.clone(), &msg)
    }

    // cw20-allowances
    #[track_caller]
    pub fn increase_allowance(
        &self,
        app: &mut App,
        sender: &Addr,
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    ) -> Result<(), ContractError> {
        let msg = cw20_allowances::ExecMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        };

        app.execute_contract(sender.clone(), self.0.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn decrease_allowance(
        &self,
        app: &mut App,
        sender: &Addr,
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    ) -> Result<(), ContractError> {
        let msg = cw20_allowances::ExecMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        };

        app.execute_contract(sender.clone(), self.0.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn transfer_from(
        &self,
        app: &mut App,
        sender: &Addr,
        owner: String,
        recipient: String,
        amount: Uint128,
    ) -> Result<(), ContractError> {
        let msg = cw20_allowances::ExecMsg::TransferFrom {
            owner,
            recipient,
            amount,
        };

        app.execute_contract(sender.clone(), self.0.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn send_from(
        &self,
        app: &mut App,
        sender: &Addr,
        owner: String,
        contract: String,
        amount: Uint128,
        msg: Binary,
    ) -> Result<(), ContractError> {
        let msg = cw20_allowances::ExecMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        };

        app.execute_contract(sender.clone(), self.0.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn burn_from(
        &self,
        app: &mut App,
        sender: &Addr,
        owner: String,
        amount: Uint128,
    ) -> Result<(), ContractError> {
        let msg = cw20_allowances::ExecMsg::BurnFrom { owner, amount };

        app.execute_contract(sender.clone(), self.0.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    pub fn allowance(
        &self,
        app: &App,
        owner: String,
        spender: String,
    ) -> StdResult<AllowanceResponse> {
        let msg = cw20_allowances::Cw20AllowancesQueryMsg::Allowance { owner, spender };

        app.wrap().query_wasm_smart(self.0.clone(), &msg)
    }

    pub fn all_allowances(
        &self,
        app: &App,
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<AllAllowancesResponse> {
        let msg = cw20_allowances::Cw20AllowancesQueryMsg::AllAllowances {
            owner,
            start_after,
            limit,
        };

        app.wrap().query_wasm_smart(self.0.clone(), &msg)
    }

    pub fn all_spender_allowances(
        &self,
        app: &App,
        spender: String,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<AllSpenderAllowancesResponse> {
        let msg = cw20_allowances::Cw20AllowancesQueryMsg::AllSpenderAllowances {
            spender,
            start_after,
            limit,
        };

        app.wrap().query_wasm_smart(self.0.clone(), &msg)
    }
}

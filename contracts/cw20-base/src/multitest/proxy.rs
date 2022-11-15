use cosmwasm_std::{Addr, Binary, StdResult, Uint128};

use cw20_allowances::responses::{
    AllAllowancesResponse, AllSpenderAllowancesResponse, AllowanceResponse,
};
use cw20_marketing::responses::{DownloadLogoResponse, MarketingInfoResponse};
use cw20_marketing::Logo;
use cw20_minting::responses::MinterResponse;
use cw_multi_test::{App, Executor};
use cw_utils::Expiration;

use crate::contract::{Cw20Base, ExecMsg, InstantiateMsg, InstantiateMsgData, QueryMsg};
use crate::error::ContractError;
use crate::responses::{BalanceResponse, TokenInfoResponse};

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
    // cw20-base
    #[track_caller]
    pub fn transfer(
        &self,
        app: &mut App,
        sender: &Addr,
        recipient: String,
        amount: Uint128,
    ) -> Result<(), ContractError> {
        let msg = ExecMsg::Transfer { recipient, amount };

        app.execute_contract(sender.clone(), self.0.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn burn(&self, app: &mut App, sender: &Addr, amount: Uint128) -> Result<(), ContractError> {
        let msg = ExecMsg::Burn { amount };

        app.execute_contract(sender.clone(), self.0.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn send(
        &self,
        app: &mut App,
        sender: &Addr,
        contract: String,
        amount: Uint128,
        msg: Binary,
    ) -> Result<(), ContractError> {
        let msg = ExecMsg::Send {
            contract,
            amount,
            msg,
        };

        app.execute_contract(sender.clone(), self.0.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    pub fn balance(&self, app: &App, address: String) -> StdResult<BalanceResponse> {
        let msg = QueryMsg::Balance { address };

        app.wrap().query_wasm_smart(self.0.clone(), &msg)
    }

    pub fn token_info(&self, app: &App, address: String) -> StdResult<TokenInfoResponse> {
        let msg = QueryMsg::TokenInfo {};

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
        let msg = cw20_allowances::QueryMsg::Allowance { owner, spender };

        app.wrap().query_wasm_smart(self.0.clone(), &msg)
    }

    pub fn all_allowances(
        &self,
        app: &App,
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<AllAllowancesResponse> {
        let msg = cw20_allowances::QueryMsg::AllAllowances {
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
        let msg = cw20_allowances::QueryMsg::AllSpenderAllowances {
            spender,
            start_after,
            limit,
        };

        app.wrap().query_wasm_smart(self.0.clone(), &msg)
    }

    // cw20-minting
    fn mint(
        &self,
        app: &mut App,
        sender: &Addr,
        recipient: String,
        amount: Uint128,
    ) -> Result<(), ContractError> {
        let msg = cw20_minting::ExecMsg::Mint { recipient, amount };

        app.execute_contract(sender.clone(), self.0.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    fn update_minter(
        &self,
        app: &mut App,
        sender: &Addr,
        new_minter: Option<String>,
    ) -> Result<(), ContractError> {
        let msg = cw20_minting::ExecMsg::UpdateMinter { new_minter };

        app.execute_contract(sender.clone(), self.0.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    pub fn minter(&self, app: &App) -> StdResult<Option<MinterResponse>> {
        let msg = cw20_minting::QueryMsg::Minter {};

        app.wrap().query_wasm_smart(self.0.clone(), &msg)
    }

    // cw20-marketing
    fn update_marketing(
        &self,
        app: &mut App,
        sender: &Addr,
        project: Option<String>,
        description: Option<String>,
        marketing: Option<String>,
    ) -> Result<(), ContractError> {
        let msg = cw20_marketing::ExecMsg::UpdateMarketing {
            project,
            description,
            marketing,
        };

        app.execute_contract(sender.clone(), self.0.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    fn update_logo(&self, app: &mut App, sender: &Addr, logo: Logo) -> Result<(), ContractError> {
        let msg = cw20_marketing::ExecMsg::UploadLogo { logo };

        app.execute_contract(sender.clone(), self.0.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    pub fn download_logo(&self, app: &App) -> StdResult<DownloadLogoResponse> {
        let msg = cw20_marketing::QueryMsg::DownloadLogo {};

        app.wrap().query_wasm_smart(self.0.clone(), &msg)
    }

    pub fn marketing_info(&self, app: &App) -> StdResult<MarketingInfoResponse> {
        let msg = cw20_marketing::QueryMsg::MarketingInfo {};

        app.wrap().query_wasm_smart(self.0.clone(), &msg)
    }
}

use crate::error::ContractError;
use crate::state::{MinterData, TokenInfo, BALANCES, LOGO, MARKETING_INFO, TOKEN_INFO};
use crate::validation::{validate_accounts, validate_msg, verify_logo};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw20::{
    BalanceResponse, Cw20Coin, Cw20ReceiveMsg, Logo, LogoInfo, MarketingInfoResponse,
    MinterResponse, TokenInfoResponse,
};
use sylvia::{contract, schemars};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw20-base";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cw_serde]
pub struct InstantiateMarketingInfo {
    pub project: Option<String>,
    pub description: Option<String>,
    pub marketing: Option<String>,
    pub logo: Option<Logo>,
}

#[cw_serde]
pub struct InstantiateMsgData {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Vec<Cw20Coin>,
    pub mint: Option<MinterResponse>,
    pub marketing: Option<InstantiateMarketingInfo>,
}

pub struct Cw20Base {}

#[contract]
impl Cw20Base {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn create_accounts(
        deps: &mut DepsMut,
        accounts: &[Cw20Coin],
    ) -> Result<Uint128, ContractError> {
        validate_accounts(accounts)?;

        let mut total_supply = Uint128::zero();
        for row in accounts {
            let address = deps.api.addr_validate(&row.address)?;
            BALANCES.save(deps.storage, &address, &row.amount)?;
            total_supply += row.amount;
        }

        Ok(total_supply)
    }

    #[msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        data: InstantiateMsgData,
    ) -> Result<Response, ContractError> {
        let (mut deps, ..) = ctx;
        let InstantiateMsgData {
            name,
            symbol,
            decimals,
            initial_balances,
            mint,
            marketing,
        } = data;
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        // check valid token info
        validate_msg(&name, &symbol, decimals)?;
        // create initial accounts
        let total_supply = Cw20Base::create_accounts(&mut deps, &initial_balances)?;

        if let Some(limit) = mint.as_ref().and_then(|v| v.cap) {
            if total_supply > limit {
                return Err(StdError::generic_err("Initial supply greater than cap").into());
            }
        }

        let mint = match mint {
            Some(m) => Some(MinterData {
                minter: deps.api.addr_validate(&m.minter)?,
                cap: m.cap,
            }),
            None => None,
        };

        // store token info
        let data = TokenInfo {
            name,
            symbol,
            decimals,
            total_supply,
            mint,
        };
        TOKEN_INFO.save(deps.storage, &data)?;

        if let Some(marketing) = marketing {
            let logo = if let Some(logo) = marketing.logo {
                verify_logo(&logo)?;
                LOGO.save(deps.storage, &logo)?;

                match logo {
                    Logo::Url(url) => Some(LogoInfo::Url(url)),
                    Logo::Embedded(_) => Some(LogoInfo::Embedded),
                }
            } else {
                None
            };

            let data = MarketingInfoResponse {
                project: marketing.project,
                description: marketing.description,
                marketing: marketing
                    .marketing
                    .map(|addr| deps.api.addr_validate(&addr))
                    .transpose()?,
                logo,
            };
            MARKETING_INFO.save(deps.storage, &data)?;
        }

        Ok(Response::default())
    }

    /// Transfer is a base message to move tokens to another account without triggering actions
    #[msg(exec)]
    fn transfer(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        recipient: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let (deps, _, info) = ctx;

        if amount == Uint128::zero() {
            return Err(ContractError::InvalidZeroAmount {});
        }

        let rcpt_addr = deps.api.addr_validate(&recipient)?;

        BALANCES.update(
            deps.storage,
            &info.sender,
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_sub(amount)?)
            },
        )?;
        BALANCES.update(
            deps.storage,
            &rcpt_addr,
            |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
        )?;

        let res = Response::new()
            .add_attribute("action", "transfer")
            .add_attribute("from", info.sender)
            .add_attribute("to", recipient)
            .add_attribute("amount", amount);
        Ok(res)
    }

    /// Burn is a base message to destroy tokens forever
    #[msg(exec)]
    fn burn(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let (deps, _, info) = ctx;

        if amount == Uint128::zero() {
            return Err(ContractError::InvalidZeroAmount {});
        }

        // lower balance
        BALANCES.update(
            deps.storage,
            &info.sender,
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_sub(amount)?)
            },
        )?;
        // reduce total_supply
        TOKEN_INFO.update(deps.storage, |mut info| -> StdResult<_> {
            info.total_supply = info.total_supply.checked_sub(amount)?;
            Ok(info)
        })?;

        let res = Response::new()
            .add_attribute("action", "burn")
            .add_attribute("from", info.sender)
            .add_attribute("amount", amount);
        Ok(res)
    }

    /// Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    #[msg(exec)]
    fn send(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        contract: String,
        amount: Uint128,
        msg: Binary,
    ) -> Result<Response, ContractError> {
        let (deps, _, info) = ctx;

        if amount == Uint128::zero() {
            return Err(ContractError::InvalidZeroAmount {});
        }

        let rcpt_addr = deps.api.addr_validate(&contract)?;

        // move the tokens to the contract
        BALANCES.update(
            deps.storage,
            &info.sender,
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_sub(amount)?)
            },
        )?;
        BALANCES.update(
            deps.storage,
            &rcpt_addr,
            |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
        )?;

        let res = Response::new()
            .add_attribute("action", "send")
            .add_attribute("from", &info.sender)
            .add_attribute("to", &contract)
            .add_attribute("amount", amount)
            .add_message(
                Cw20ReceiveMsg {
                    sender: info.sender.into(),
                    amount,
                    msg,
                }
                .into_cosmos_msg(contract)?,
            );
        Ok(res)
    }

    /// Returns the current balance of the given address, 0 if unset.
    #[msg(query)]
    fn balance(&self, ctx: (Deps, Env), address: String) -> StdResult<BalanceResponse> {
        let (deps, _) = ctx;
        let address = deps.api.addr_validate(&address)?;
        let balance = BALANCES
            .may_load(deps.storage, &address)?
            .unwrap_or_default();
        Ok(BalanceResponse { balance })
    }

    /// Returns metadata on the contract - name, decimals, supply, etc.
    #[msg(query)]
    fn token_info(&self, ctx: (Deps, Env)) -> StdResult<TokenInfoResponse> {
        let (deps, _) = ctx;

        let info = TOKEN_INFO.load(deps.storage)?;
        let res = TokenInfoResponse {
            name: info.name,
            symbol: info.symbol,
            decimals: info.decimals,
            total_supply: info.total_supply,
        };
        Ok(res)
    }
}

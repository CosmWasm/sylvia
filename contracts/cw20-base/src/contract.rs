use crate::error::ContractError;
use crate::state::{MinterData, TokenInfo, BALANCES, LOGO, MARKETING_INFO, TOKEN_INFO};
use crate::validation::{validate_accounts, validate_msg, verify_logo};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, Uint128};
use cw2::set_contract_version;
use cw20::{Cw20Coin, Logo, LogoInfo, MarketingInfoResponse, MinterResponse};
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
    // /// Transfer is a base message to move tokens to another account without triggering actions
    // #[msg(exec)]
    // fn transfer(
    //     &self,
    //     ctx: (DepsMut, Env, MessageInfo),
    //     recipient: String,
    //     amount: Uint128,
    // ) -> Result<Response, Self::Error>;

    // /// Burn is a base message to destroy tokens forever
    // #[msg(exec)]
    // fn burn(
    //     &self,
    //     ctx: (DepsMut, Env, MessageInfo),
    //     new_minteramount: Uint128,
    // ) -> Result<Response, Self::Error>;

    // /// Send is a base message to transfer tokens to a contract and trigger an action
    // /// on the receiving contract.
    // #[msg(exec)]
    // fn send(
    //     &self,
    //     ctx: (DepsMut, Env, MessageInfo),
    //     contract: String,
    //     amount: Uint128,
    //     msg: Binary,
    // ) -> Result<Response, Self::Error>;

    // /// Returns the current balance of the given address, 0 if unset.
    // #[msg(query)]
    // fn balance(&self, ctx: (Deps, Env), address: String) -> StdResult<BalanceResponse>;

    // /// Returns metadata on the contract - name, decimals, supply, etc.
    // #[msg(query)]
    // fn token_info(&self, ctx: (Deps, Env)) -> StdResult<TokenInfoResponse>;
}

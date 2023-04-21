use crate::error::ContractError;
use crate::responses::{BalanceResponse, Cw20Coin, Cw20ReceiveMsg, TokenInfoResponse};
use crate::validation::{validate_accounts, validate_msg, verify_logo};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    ensure, Addr, Binary, BlockInfo, DepsMut, Order, Response, StdError, StdResult, Storage,
    Uint128,
};
use cw2::set_contract_version;
use cw20_allowances::responses::AllowanceResponse;
use cw20_marketing::responses::{LogoInfo, MarketingInfoResponse};
use cw20_marketing::Logo;
use cw20_minting::responses::MinterResponse;
use cw_storage_plus::{Item, Map};
use cw_utils::ensure_from_older_version;
use sylvia::types::{ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx};
use sylvia::{contract, schemars};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw20-base";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cw_serde]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
    pub mint: Option<MinterData>,
}

impl TokenInfo {
    pub fn get_cap(&self) -> Option<Uint128> {
        self.mint.as_ref().and_then(|v| v.cap)
    }
}

#[cw_serde]
pub struct MinterData {
    pub minter: Addr,
    /// cap is how many more tokens can be issued by the minter
    pub cap: Option<Uint128>,
}

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

pub struct Cw20Base<'a> {
    pub(crate) token_info: Item<'static, TokenInfo>,
    pub(crate) marketing_info: Item<'static, MarketingInfoResponse>,
    pub(crate) logo: Item<'static, Logo>,
    pub(crate) balances: Map<'static, &'a Addr, Uint128>,
    pub(crate) allowances: Map<'static, (&'a Addr, &'a Addr), AllowanceResponse>,
    // TODO: After https://github.com/CosmWasm/cw-plus/issues/670 is implemented, replace this with a `MultiIndex` over `ALLOWANCES`
    pub(crate) allowances_spender: Map<'static, (&'a Addr, &'a Addr), AllowanceResponse>,
}

#[contract(error=ContractError)]
#[messages(cw20_allowances as Allowances)]
#[messages(cw20_marketing as Marketing)]
#[messages(cw20_minting as Minting)]
impl Cw20Base<'_> {
    pub const fn new() -> Self {
        Self {
            token_info: Item::new("token_info"),
            marketing_info: Item::new("marketing_info"),
            logo: Item::new("logo"),
            balances: Map::new("balances"),
            allowances: Map::new("allowances"),
            allowances_spender: Map::new("allowances_spender"),
        }
    }

    pub fn create_accounts(
        &self,
        deps: &mut DepsMut,
        accounts: &[Cw20Coin],
    ) -> Result<Uint128, ContractError> {
        validate_accounts(accounts)?;

        let mut total_supply = Uint128::zero();
        for row in accounts {
            let address = deps.api.addr_validate(&row.address)?;
            self.balances.save(deps.storage, &address, &row.amount)?;
            total_supply += row.amount;
        }

        Ok(total_supply)
    }

    // this can be used to update a lower allowance - call bucket.update with proper keys
    pub fn deduct_allowance(
        &self,
        storage: &mut dyn Storage,
        owner: &Addr,
        spender: &Addr,
        block: &BlockInfo,
        amount: Uint128,
    ) -> Result<AllowanceResponse, ContractError> {
        let update_fn = |current: Option<AllowanceResponse>| -> _ {
            let allowance = current.ok_or(ContractError::NoAllowance)?;

            if !allowance.expires.is_expired(block) {
                // deduct the allowance if enough
                let expires = allowance.expires;
                let allowance = allowance
                    .allowance
                    .checked_sub(amount)
                    .map_err(StdError::overflow)?;
                Ok(AllowanceResponse { allowance, expires })
            } else {
                Err(ContractError::Expired)
            }
        };
        self.allowances
            .update(storage, (owner, spender), update_fn)?;
        self.allowances_spender
            .update(storage, (spender, owner), update_fn)
    }

    #[msg(instantiate)]
    pub fn instantiate(
        &self,
        mut ctx: InstantiateCtx,
        data: InstantiateMsgData,
    ) -> Result<Response, ContractError> {
        let InstantiateMsgData {
            name,
            symbol,
            decimals,
            initial_balances,
            mint,
            marketing,
        } = data;
        set_contract_version(ctx.deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        // check valid token info
        validate_msg(&name, &symbol, decimals)?;
        // create initial accounts
        let total_supply = self.create_accounts(&mut ctx.deps, &initial_balances)?;

        ensure!(
            !matches!(
                mint.as_ref().and_then(|v| v.cap), Some(limit) if total_supply > limit
            ),
            StdError::generic_err("Initial supply greater than cap")
        );

        let mint = match mint {
            Some(m) => Some(MinterData {
                minter: ctx.deps.api.addr_validate(&m.minter)?,
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
        self.token_info.save(ctx.deps.storage, &data)?;

        if let Some(marketing) = marketing {
            let logo = if let Some(logo) = marketing.logo {
                verify_logo(&logo)?;
                self.logo.save(ctx.deps.storage, &logo)?;

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
                    .map(|addr| ctx.deps.api.addr_validate(&addr))
                    .transpose()?,
                logo,
            };
            self.marketing_info.save(ctx.deps.storage, &data)?;
        }

        Ok(Response::default())
    }

    /// Transfer is a base message to move tokens to another account without triggering actions
    #[msg(exec)]
    fn transfer(
        &self,
        ctx: ExecCtx,
        recipient: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        ensure!(amount != Uint128::zero(), ContractError::InvalidZeroAmount);

        let rcpt_addr = ctx.deps.api.addr_validate(&recipient)?;

        self.balances
            .update(ctx.deps.storage, &ctx.info.sender, |balance| {
                Ok::<_, StdError>(balance.unwrap_or_default().checked_sub(amount)?)
            })?;
        self.balances
            .update(ctx.deps.storage, &rcpt_addr, |balance| {
                Ok::<_, StdError>(balance.unwrap_or_default().checked_add(amount)?)
            })?;

        let res = Response::new()
            .add_attribute("action", "transfer")
            .add_attribute("from", ctx.info.sender)
            .add_attribute("to", recipient)
            .add_attribute("amount", amount);
        Ok(res)
    }

    /// Burn is a base message to destroy tokens forever
    #[msg(exec)]
    fn burn(&self, ctx: ExecCtx, amount: Uint128) -> Result<Response, ContractError> {
        ensure!(amount != Uint128::zero(), ContractError::InvalidZeroAmount);

        // lower balance
        self.balances
            .update(ctx.deps.storage, &ctx.info.sender, |balance| {
                Ok::<_, StdError>(balance.unwrap_or_default().checked_sub(amount)?)
            })?;
        // reduce total_supply
        self.token_info
            .update(ctx.deps.storage, |mut info| -> StdResult<_> {
                info.total_supply = info.total_supply.checked_sub(amount)?;
                Ok(info)
            })?;

        let res = Response::new()
            .add_attribute("action", "burn")
            .add_attribute("from", ctx.info.sender)
            .add_attribute("amount", amount);
        Ok(res)
    }

    /// Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    #[msg(exec)]
    fn send(
        &self,
        ctx: ExecCtx,
        contract: String,
        amount: Uint128,
        msg: Binary,
    ) -> Result<Response, ContractError> {
        ensure!(amount != Uint128::zero(), ContractError::InvalidZeroAmount);

        let rcpt_addr = ctx.deps.api.addr_validate(&contract)?;

        // move the tokens to the contract
        self.balances
            .update(ctx.deps.storage, &ctx.info.sender, |balance| {
                Ok::<_, StdError>(balance.unwrap_or_default().checked_sub(amount)?)
            })?;
        self.balances
            .update(ctx.deps.storage, &rcpt_addr, |balance| {
                Ok::<_, StdError>(balance.unwrap_or_default().checked_add(amount)?)
            })?;
        let res = Response::new()
            .add_attribute("action", "send")
            .add_attribute("from", &ctx.info.sender)
            .add_attribute("to", &contract)
            .add_attribute("amount", amount);

        let msg = Cw20ReceiveMsg {
            sender: ctx.info.sender.into(),
            amount,
            msg,
        }
        .into_cosmos_msg(contract)?;

        let res = res.add_message(msg);
        Ok(res)
    }

    /// Returns the current balance of the given address, 0 if unset.
    #[msg(query)]
    fn balance(&self, ctx: QueryCtx, address: String) -> StdResult<BalanceResponse> {
        let address = ctx.deps.api.addr_validate(&address)?;
        let balance = self
            .balances
            .may_load(ctx.deps.storage, &address)?
            .unwrap_or_default();
        Ok(BalanceResponse { balance })
    }

    /// Returns metadata on the contract - name, decimals, supply, etc.
    #[msg(query)]
    fn token_info(&self, ctx: QueryCtx) -> StdResult<TokenInfoResponse> {
        let info = self.token_info.load(ctx.deps.storage)?;
        let res = TokenInfoResponse {
            name: info.name,
            symbol: info.symbol,
            decimals: info.decimals,
            total_supply: info.total_supply,
        };
        Ok(res)
    }

    #[msg(migrate)]
    fn migrate(&self, ctx: MigrateCtx) -> Result<Response, ContractError> {
        let original_version =
            ensure_from_older_version(ctx.deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        if original_version < "0.14.0".parse::<semver::Version>().unwrap() {
            // Build reverse map of allowances per spender
            let data = self
                .allowances
                .range(ctx.deps.storage, None, None, Order::Ascending)
                .collect::<StdResult<Vec<_>>>()?;
            for ((owner, spender), allowance) in data {
                self.allowances_spender
                    .save(ctx.deps.storage, (&spender, &owner), &allowance)?;
            }
        }
        Ok(Response::default())
    }
}

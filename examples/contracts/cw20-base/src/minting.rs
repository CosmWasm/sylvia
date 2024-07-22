use crate::contract::{Cw20Base, MinterData};
use crate::error::ContractError;
use cosmwasm_std::{Response, StdResult, Uint128};
use cw20_minting::responses::MinterResponse;
use cw20_minting::Cw20Minting;
use sylvia::types::{ExecCtx, QueryCtx};

impl Cw20Minting for Cw20Base {
    type Error = ContractError;

    fn mint(
        &self,
        ctx: ExecCtx,
        recipient: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        if amount == Uint128::zero() {
            return Err(ContractError::InvalidZeroAmount);
        }

        let mut config = self
            .token_info
            .may_load(ctx.deps.storage)?
            .ok_or(ContractError::Unauthorized)?;

        if config
            .mint
            .as_ref()
            .ok_or(ContractError::Unauthorized)?
            .minter
            != ctx.info.sender
        {
            return Err(ContractError::Unauthorized);
        }

        // update supply and enforce cap
        config.total_supply += amount;
        if let Some(limit) = config.get_cap() {
            if config.total_supply > limit {
                return Err(ContractError::CannotExceedCap);
            }
        }
        self.token_info.save(ctx.deps.storage, &config)?;

        // add amount to recipient balance
        let rcpt_addr = ctx.deps.api.addr_validate(&recipient)?;
        self.balances.update(
            ctx.deps.storage,
            &rcpt_addr,
            |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
        )?;

        let res = Response::new()
            .add_attribute("action", "mint")
            .add_attribute("to", recipient)
            .add_attribute("amount", amount);
        Ok(res)
    }

    fn update_minter(
        &self,
        ctx: ExecCtx,
        new_minter: Option<String>,
    ) -> Result<Response, Self::Error> {
        let mut config = self
            .token_info
            .may_load(ctx.deps.storage)?
            .ok_or(ContractError::Unauthorized)?;

        let mint = config.mint.as_ref().ok_or(ContractError::Unauthorized)?;
        if mint.minter != ctx.info.sender {
            return Err(ContractError::Unauthorized);
        }

        let minter_data = new_minter
            .map(|new_minter| ctx.deps.api.addr_validate(&new_minter))
            .transpose()?
            .map(|minter| MinterData {
                minter,
                cap: mint.cap,
            });

        config.mint = minter_data;

        self.token_info.save(ctx.deps.storage, &config)?;

        let resp = Response::new()
            .add_attribute("action", "update_minter")
            .add_attribute(
                "new_minter",
                config
                    .mint
                    .map(|m| m.minter.into_string())
                    .unwrap_or_else(|| "None".to_string()),
            );
        Ok(resp)
    }

    fn minter(&self, ctx: QueryCtx) -> StdResult<Option<MinterResponse>> {
        let meta = self.token_info.load(ctx.deps.storage)?;
        let minter = match meta.mint {
            Some(m) => Some(MinterResponse {
                minter: m.minter.into(),
                cap: m.cap,
            }),
            None => None,
        };
        Ok(minter)
    }
}

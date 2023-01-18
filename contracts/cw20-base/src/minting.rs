use crate::contract::{Cw20Base, MinterData};
use crate::error::ContractError;
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128};
use cw20_minting::responses::MinterResponse;
use cw20_minting::Cw20Minting;
use sylvia::contract;

#[contract]
impl Cw20Minting for Cw20Base<'_> {
    type Error = ContractError;

    fn mint(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        recipient: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let (deps, _, info) = ctx;

        if amount == Uint128::zero() {
            return Err(ContractError::InvalidZeroAmount {});
        }

        let mut config = self
            .token_info
            .may_load(deps.storage)?
            .ok_or(ContractError::Unauthorized {})?;

        if config
            .mint
            .as_ref()
            .ok_or(ContractError::Unauthorized {})?
            .minter
            != info.sender
        {
            return Err(ContractError::Unauthorized {});
        }

        // update supply and enforce cap
        config.total_supply += amount;
        if let Some(limit) = config.get_cap() {
            if config.total_supply > limit {
                return Err(ContractError::CannotExceedCap {});
            }
        }
        self.token_info.save(deps.storage, &config)?;

        // add amount to recipient balance
        let rcpt_addr = deps.api.addr_validate(&recipient)?;
        self.balances.update(
            deps.storage,
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
        ctx: (DepsMut, Env, MessageInfo),
        new_minter: Option<String>,
    ) -> Result<Response, Self::Error> {
        let (deps, _, info) = ctx;

        let mut config = self
            .token_info
            .may_load(deps.storage)?
            .ok_or(ContractError::Unauthorized {})?;

        let mint = config.mint.as_ref().ok_or(ContractError::Unauthorized {})?;
        if mint.minter != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        let minter_data = new_minter
            .map(|new_minter| deps.api.addr_validate(&new_minter))
            .transpose()?
            .map(|minter| MinterData {
                minter,
                cap: mint.cap,
            });

        config.mint = minter_data;

        self.token_info.save(deps.storage, &config)?;

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

    fn minter(&self, ctx: (Deps, Env)) -> StdResult<Option<MinterResponse>> {
        let (deps, _) = ctx;

        let meta = self.token_info.load(deps.storage)?;
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

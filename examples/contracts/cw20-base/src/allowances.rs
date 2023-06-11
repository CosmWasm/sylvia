use cosmwasm_std::{Addr, Binary, Order, Response, StdError, StdResult, Uint128};
use cw20_allowances::responses::{
    AllAccountsResponse, AllAllowancesResponse, AllSpenderAllowancesResponse, AllowanceInfo,
    AllowanceResponse, SpenderAllowanceInfo,
};
use cw20_allowances::Cw20Allowances;
use cw_storage_plus::{Bound, Bounder};
use cw_utils::Expiration;
use sylvia::contract;
use sylvia::types::{ExecCtx, QueryCtx};

use crate::contract::Cw20Base;
use crate::error::ContractError;
use crate::responses::Cw20ReceiveMsg;

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

#[contract(module=crate::contract)]
#[messages(cw20_allowances as Cw20Allowances)]
impl Cw20Allowances for Cw20Base<'_> {
    type Error = ContractError;

    /// Allows spender to access an additional amount tokens from the owner's (env.sender) account.
    /// If expires is Some(), overwrites current allowance expiration with this one.
    #[msg(exec)]
    fn increase_allowance(
        &self,
        ctx: ExecCtx,
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    ) -> Result<Response, Self::Error> {
        let spender_addr = ctx.deps.api.addr_validate(&spender)?;
        if spender_addr == ctx.info.sender {
            return Err(ContractError::CannotSetOwnAccount);
        }

        let update_fn = |allow: Option<AllowanceResponse>| -> Result<_, _> {
            let allow = allow.unwrap_or_default();
            let allowance = allow.allowance + amount;
            match expires {
                Some(expires) if !expires.is_expired(&ctx.env.block) => {
                    Ok(AllowanceResponse { allowance, expires })
                }
                None => Ok(AllowanceResponse { allowance, ..allow }),
                _ => Err(ContractError::InvalidExpiration),
            }
        };
        self.allowances.update(
            ctx.deps.storage,
            (&ctx.info.sender, &spender_addr),
            update_fn,
        )?;
        self.allowances_spender.update(
            ctx.deps.storage,
            (&spender_addr, &ctx.info.sender),
            update_fn,
        )?;

        let res = Response::new()
            .add_attribute("action", "increase_allowance")
            .add_attribute("owner", ctx.info.sender)
            .add_attribute("spender", spender)
            .add_attribute("amount", amount);
        Ok(res)
    }

    /// Lowers the spender's access of tokens from the owner's (env.sender) account by amount.
    /// If expires is Some(), overwrites current allowance expiration with this one.
    #[msg(exec)]
    fn decrease_allowance(
        &self,
        ctx: ExecCtx,
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    ) -> Result<Response, Self::Error> {
        let spender_addr = Addr::unchecked(&spender);
        if spender_addr == ctx.info.sender {
            return Err(ContractError::CannotSetOwnAccount);
        }

        let key = (&ctx.info.sender, &spender_addr);

        let reverse = |(a, b)| (b, a);

        // load value and delete if it hits 0, or update otherwise
        let mut allowance = self.allowances.load(ctx.deps.storage, key)?;
        if amount < allowance.allowance {
            // update the new amount
            allowance.allowance = allowance
                .allowance
                .checked_sub(amount)
                .map_err(StdError::overflow)?;
            if let Some(exp) = expires {
                if exp.is_expired(&ctx.env.block) {
                    return Err(ContractError::InvalidExpiration);
                }
                allowance.expires = exp;
            }
            self.allowances.save(ctx.deps.storage, key, &allowance)?;
            self.allowances_spender
                .save(ctx.deps.storage, reverse(key), &allowance)?;
        } else {
            self.allowances.remove(ctx.deps.storage, key);
            self.allowances_spender
                .remove(ctx.deps.storage, reverse(key));
        }

        let res = Response::new()
            .add_attribute("action", "decrease_allowance")
            .add_attribute("owner", ctx.info.sender)
            .add_attribute("spender", spender)
            .add_attribute("amount", amount);
        Ok(res)
    }

    /// Transfers amount tokens from owner -> recipient
    /// if `env.sender` has sufficient pre-approval.
    #[msg(exec)]
    fn transfer_from(
        &self,
        ctx: ExecCtx,
        owner: String,
        recipient: String,
        amount: Uint128,
    ) -> Result<Response, Self::Error> {
        let rcpt_addr = ctx.deps.api.addr_validate(&recipient)?;
        let owner_addr = ctx.deps.api.addr_validate(&owner)?;

        // Avoid doing state update in case of self to self transfer
        if rcpt_addr == owner_addr {
            let resp = Response::new()
                .add_attribute("action", "transfer_from")
                .add_attribute("from", owner)
                .add_attribute("to", recipient)
                .add_attribute("by", ctx.info.sender)
                .add_attribute("amount", amount);
            return Ok(resp);
        }

        if ctx.info.sender != owner {
            // deduct allowance before doing anything else have enough allowance
            self.deduct_allowance(
                ctx.deps.storage,
                &owner_addr,
                &ctx.info.sender,
                &ctx.env.block,
                amount,
            )?;
        }

        self.balances.update(
            ctx.deps.storage,
            &owner_addr,
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_sub(amount)?)
            },
        )?;
        self.balances.update(
            ctx.deps.storage,
            &rcpt_addr,
            |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
        )?;

        let res = Response::new()
            .add_attribute("action", "transfer_from")
            .add_attribute("from", owner)
            .add_attribute("to", recipient)
            .add_attribute("by", ctx.info.sender)
            .add_attribute("amount", amount);
        Ok(res)
    }

    /// Sends amount tokens from owner -> contract
    /// if `env.sender` has sufficient pre-approval.
    #[msg(exec)]
    fn send_from(
        &self,
        ctx: ExecCtx,
        owner: String,
        contract: String,
        amount: Uint128,
        msg: Binary,
    ) -> Result<Response, Self::Error> {
        let rcpt_addr = ctx.deps.api.addr_validate(&contract)?;
        let owner_addr = ctx.deps.api.addr_validate(&owner)?;

        // deduct allowance before doing anything else have enough allowance
        self.deduct_allowance(
            ctx.deps.storage,
            &owner_addr,
            &ctx.info.sender,
            &ctx.env.block,
            amount,
        )?;

        // move the tokens to the contract
        self.balances.update(
            ctx.deps.storage,
            &owner_addr,
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_sub(amount)?)
            },
        )?;
        self.balances.update(
            ctx.deps.storage,
            &rcpt_addr,
            |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
        )?;

        let resp = Response::new()
            .add_attribute("action", "send_from")
            .add_attribute("from", &owner)
            .add_attribute("to", &contract)
            .add_attribute("by", &ctx.info.sender)
            .add_attribute("amount", amount);

        // create a send message
        let msg = Cw20ReceiveMsg {
            sender: ctx.info.sender.into(),
            amount,
            msg,
        }
        .into_cosmos_msg(contract)?;

        let resp = resp.add_message(msg);
        Ok(resp)
    }

    /// Destroys amount of tokens forever
    #[msg(exec)]
    fn burn_from(
        &self,
        ctx: ExecCtx,
        owner: String,
        amount: Uint128,
    ) -> Result<Response, Self::Error> {
        let owner_addr = ctx.deps.api.addr_validate(&owner)?;

        // deduct allowance before doing anything else have enough allowance
        self.deduct_allowance(
            ctx.deps.storage,
            &owner_addr,
            &ctx.info.sender,
            &ctx.env.block,
            amount,
        )?;

        // lower balance
        self.balances.update(
            ctx.deps.storage,
            &owner_addr,
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_sub(amount)?)
            },
        )?;
        // reduce total_supply
        self.token_info
            .update(ctx.deps.storage, |mut meta| -> StdResult<_> {
                meta.total_supply = meta.total_supply.checked_sub(amount)?;
                Ok(meta)
            })?;

        let res = Response::new()
            .add_attribute("action", "burn_from")
            .add_attribute("owner", owner)
            .add_attribute("spender", ctx.info.sender)
            .add_attribute("amount", amount);
        Ok(res)
    }

    /// Returns how much spender can use from owner account, 0 if unset.
    #[msg(query)]
    fn allowance(
        &self,
        ctx: QueryCtx,
        owner: String,
        spender: String,
    ) -> StdResult<AllowanceResponse> {
        let owner_addr = ctx.deps.api.addr_validate(&owner)?;
        let spender_addr = ctx.deps.api.addr_validate(&spender)?;
        let allowance = self
            .allowances
            .may_load(ctx.deps.storage, (&owner_addr, &spender_addr))?
            .unwrap_or_default();
        Ok(allowance)
    }

    /// Returns all allowances this owner has approved. Supports pagination.
    #[msg(query)]
    fn all_allowances(
        &self,
        ctx: QueryCtx,
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<AllAllowancesResponse> {
        let owner_addr = ctx.deps.api.addr_validate(&owner)?;
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.map(|s| Bound::ExclusiveRaw(s.into_bytes()));

        let allowances = self
            .allowances
            .prefix(&owner_addr)
            .range(ctx.deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| {
                item.map(|(addr, allow)| AllowanceInfo {
                    spender: addr.into(),
                    allowance: allow.allowance,
                    expires: allow.expires,
                })
            })
            .collect::<StdResult<_>>()?;
        Ok(AllAllowancesResponse { allowances })
    }

    /// Returns all allowances this spender has been granted. Supports pagination.
    #[msg(query)]
    fn all_spender_allowances(
        &self,
        ctx: QueryCtx,
        spender: String,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<AllSpenderAllowancesResponse> {
        let spender_addr = ctx.deps.api.addr_validate(&spender)?;
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start_after = start_after.map(Addr::unchecked);
        let start = start_after.as_ref().and_then(Bounder::exclusive_bound);

        let allowances = self
            .allowances_spender
            .prefix(&spender_addr)
            .range(ctx.deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| {
                item.map(|(addr, allow)| SpenderAllowanceInfo {
                    owner: addr.into(),
                    allowance: allow.allowance,
                    expires: allow.expires,
                })
            })
            .collect::<StdResult<_>>()?;
        Ok(AllSpenderAllowancesResponse { allowances })
    }

    /// Returns all allowances this spender has been granted. Supports pagination.
    #[msg(query)]
    fn all_accounts(
        &self,
        ctx: QueryCtx,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<AllAccountsResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

        let accounts = self
            .balances
            .keys(ctx.deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| item.map(Into::into))
            .collect::<StdResult<_>>()?;

        Ok(AllAccountsResponse { accounts })
    }
}

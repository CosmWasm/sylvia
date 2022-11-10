use cosmwasm_std::{
    attr, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdError, StdResult,
    Uint128,
};
use cw20_allowances::responses::{
    AllAllowancesResponse, AllSpenderAllowancesResponse, AllowanceInfo, AllowanceResponse,
    SpenderAllowanceInfo,
};
use cw20_allowances::Cw20Allowances;
use cw_storage_plus::Bound;
use cw_utils::Expiration;

use crate::contract::Cw20Base;
use crate::error::ContractError;
use crate::responses::Cw20ReceiveMsg;

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

impl Cw20Allowances for Cw20Base<'_> {
    type Error = ContractError;

    /// Allows spender to access an additional amount tokens from the owner's (env.sender) account.
    /// If expires is Some(), overwrites current allowance expiration with this one.
    fn increase_allowance(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    ) -> Result<Response, Self::Error> {
        let (deps, env, info) = ctx;

        let spender_addr = deps.api.addr_validate(&spender)?;
        if spender_addr == info.sender {
            return Err(ContractError::CannotSetOwnAccount {});
        }

        let update_fn = |allow: Option<AllowanceResponse>| -> Result<_, _> {
            let allow = allow.unwrap_or_default();
            let allowance = allow.allowance + amount;
            match expires {
                Some(expires) if !expires.is_expired(&env.block) => {
                    Ok(AllowanceResponse { allowance, expires })
                }
                None => Ok(AllowanceResponse { allowance, ..allow }),
                _ => Err(ContractError::InvalidExpiration {}),
            }
        };
        self.allowances
            .update(deps.storage, (&info.sender, &spender_addr), update_fn)?;
        self.allowances_spender
            .update(deps.storage, (&spender_addr, &info.sender), update_fn)?;

        let res = Response::new().add_attributes(vec![
            attr("action", "increase_allowance"),
            attr("owner", info.sender),
            attr("spender", spender),
            attr("amount", amount),
        ]);
        Ok(res)
    }

    /// Lowers the spender's access of tokens from the owner's (env.sender) account by amount.
    /// If expires is Some(), overwrites current allowance expiration with this one.
    fn decrease_allowance(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    ) -> Result<Response, Self::Error> {
        let (deps, env, info) = ctx;

        let spender_addr = deps.api.addr_validate(&spender)?;
        if spender_addr == info.sender {
            return Err(ContractError::CannotSetOwnAccount {});
        }

        let key = (&info.sender, &spender_addr);

        fn reverse<'a>(t: (&'a Addr, &'a Addr)) -> (&'a Addr, &'a Addr) {
            (t.1, t.0)
        }

        // load value and delete if it hits 0, or update otherwise
        let mut allowance = self.allowances.load(deps.storage, key)?;
        if amount < allowance.allowance {
            // update the new amount
            allowance.allowance = allowance
                .allowance
                .checked_sub(amount)
                .map_err(StdError::overflow)?;
            if let Some(exp) = expires {
                if exp.is_expired(&env.block) {
                    return Err(ContractError::InvalidExpiration {});
                }
                allowance.expires = exp;
            }
            self.allowances.save(deps.storage, key, &allowance)?;
            self.allowances_spender
                .save(deps.storage, reverse(key), &allowance)?;
        } else {
            self.allowances.remove(deps.storage, key);
            self.allowances_spender.remove(deps.storage, reverse(key));
        }

        let res = Response::new().add_attributes(vec![
            attr("action", "decrease_allowance"),
            attr("owner", info.sender),
            attr("spender", spender),
            attr("amount", amount),
        ]);
        Ok(res)
    }

    /// Transfers amount tokens from owner -> recipient
    /// if `env.sender` has sufficient pre-approval.
    fn transfer_from(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        owner: String,
        recipient: String,
        amount: Uint128,
    ) -> Result<Response, Self::Error> {
        let (deps, env, info) = ctx;

        let rcpt_addr = deps.api.addr_validate(&recipient)?;
        let owner_addr = deps.api.addr_validate(&owner)?;

        // Avoid doing state update in case of self to self transfer
        if rcpt_addr == owner_addr {
            return Ok(Response::new().add_attributes(vec![
                attr("action", "transfer_from"),
                attr("from", owner),
                attr("to", recipient),
                attr("by", info.sender),
                attr("amount", amount),
            ]));
        }

        if info.sender != owner {
            // deduct allowance before doing anything else have enough allowance
            self.deduct_allowance(deps.storage, &owner_addr, &info.sender, &env.block, amount)?;
        }

        self.balances.update(
            deps.storage,
            &owner_addr,
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_sub(amount)?)
            },
        )?;
        self.balances.update(
            deps.storage,
            &rcpt_addr,
            |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
        )?;

        let res = Response::new().add_attributes(vec![
            attr("action", "transfer_from"),
            attr("from", owner),
            attr("to", recipient),
            attr("by", info.sender),
            attr("amount", amount),
        ]);
        Ok(res)
    }

    /// Sends amount tokens from owner -> contract
    /// if `env.sender` has sufficient pre-approval.
    fn send_from(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        owner: String,
        contract: String,
        amount: Uint128,
        msg: Binary,
    ) -> Result<Response, Self::Error> {
        let (deps, env, info) = ctx;

        let rcpt_addr = deps.api.addr_validate(&contract)?;
        let owner_addr = deps.api.addr_validate(&owner)?;

        // deduct allowance before doing anything else have enough allowance
        self.deduct_allowance(deps.storage, &owner_addr, &info.sender, &env.block, amount)?;

        // move the tokens to the contract
        self.balances.update(
            deps.storage,
            &owner_addr,
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_sub(amount)?)
            },
        )?;
        self.balances.update(
            deps.storage,
            &rcpt_addr,
            |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
        )?;

        let attrs = vec![
            attr("action", "send_from"),
            attr("from", &owner),
            attr("to", &contract),
            attr("by", &info.sender),
            attr("amount", amount),
        ];

        // create a send message
        let msg = Cw20ReceiveMsg {
            sender: info.sender.into(),
            amount,
            msg,
        }
        .into_cosmos_msg(contract)?;

        let res = Response::new().add_message(msg).add_attributes(attrs);
        Ok(res)
    }

    /// Destroys amount of tokens forever
    fn burn_from(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        owner: String,
        amount: Uint128,
    ) -> Result<Response, Self::Error> {
        let (deps, env, info) = ctx;

        let owner_addr = deps.api.addr_validate(&owner)?;

        // deduct allowance before doing anything else have enough allowance
        self.deduct_allowance(deps.storage, &owner_addr, &info.sender, &env.block, amount)?;

        // lower balance
        self.balances.update(
            deps.storage,
            &owner_addr,
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_sub(amount)?)
            },
        )?;
        // reduce total_supply
        self.token_info
            .update(deps.storage, |mut meta| -> StdResult<_> {
                meta.total_supply = meta.total_supply.checked_sub(amount)?;
                Ok(meta)
            })?;

        let res = Response::new().add_attributes(vec![
            attr("action", "burn_from"),
            attr("from", owner),
            attr("by", info.sender),
            attr("amount", amount),
        ]);
        Ok(res)
    }

    /// Returns how much spender can use from owner account, 0 if unset.
    fn allowance(
        &self,
        ctx: (Deps, Env),
        owner: String,
        spender: String,
    ) -> StdResult<AllowanceResponse> {
        let (deps, _) = ctx;

        let owner_addr = deps.api.addr_validate(&owner)?;
        let spender_addr = deps.api.addr_validate(&spender)?;
        let allowance = self
            .allowances
            .may_load(deps.storage, (&owner_addr, &spender_addr))?
            .unwrap_or_default();
        Ok(allowance)
    }

    /// Returns all allowances this owner has approved. Supports pagination.
    fn all_allowances(
        &self,
        ctx: (Deps, Env),
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<AllAllowancesResponse> {
        let (deps, _) = ctx;

        let owner_addr = deps.api.addr_validate(&owner)?;
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.map(|s| Bound::ExclusiveRaw(s.into_bytes()));

        let allowances = self
            .allowances
            .prefix(&owner_addr)
            .range(deps.storage, start, None, Order::Ascending)
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
    fn all_spender_allowances(
        &self,
        ctx: (Deps, Env),
        spender: String,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<AllSpenderAllowancesResponse> {
        let (deps, _) = ctx;

        let spender_addr = deps.api.addr_validate(&spender)?;
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.map(|s| Bound::ExclusiveRaw(s.into_bytes()));

        let allowances = self
            .allowances_spender
            .prefix(&spender_addr)
            .range(deps.storage, start, None, Order::Ascending)
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
}

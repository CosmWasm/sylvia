use cosmwasm_std::{
    ensure, ensure_ne, Addr, BankMsg, Coin, CosmosMsg, Deps, DistributionMsg, Env, Order, Response,
    StakingMsg, StdResult,
};
use cw1_whitelist::contract::Cw1WhitelistContract;
use cw2::set_contract_version;
use cw_storage_plus::{Bound, Map};
use cw_utils::Expiration;
use sylvia::types::{ExecCtx, InstantiateCtx, QueryCtx};
use sylvia::{contract, schemars};

#[cfg(not(feature = "library"))]
use sylvia::entry_points;

use crate::error::ContractError;
use crate::responses::{
    AllAllowancesResponse, AllPermissionsResponse, AllowanceInfo, PermissionsInfo,
};
use crate::state::{Allowance, Permissions};

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Default and max limits for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

pub struct Cw1SubkeysContract<'a> {
    pub(crate) whitelist: Cw1WhitelistContract<'a>,
    pub(crate) permissions: Map<'static, &'a Addr, Permissions>,
    pub(crate) allowances: Map<'static, &'a Addr, Allowance>,
}

#[cfg_attr(not(feature = "library"), entry_points)]
#[contract]
#[sv::error(ContractError)]
#[sv::messages(cw1 as Cw1)]
#[sv::messages(whitelist as Whitelist)]
impl Cw1SubkeysContract<'_> {
    pub const fn new() -> Self {
        Self {
            whitelist: Cw1WhitelistContract::new(),
            permissions: Map::new("permissions"),
            allowances: Map::new("allowances"),
        }
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(
        &self,
        mut ctx: InstantiateCtx,
        admins: Vec<String>,
        mutable: bool,
    ) -> Result<Response, ContractError> {
        let result = self.whitelist.instantiate(ctx.branch(), admins, mutable)?;
        set_contract_version(ctx.deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        Ok(result)
    }

    #[sv::msg(exec)]
    pub fn increase_allowance(
        &self,
        ctx: ExecCtx,
        spender: String,
        amount: Coin,
        expires: Option<Expiration>,
    ) -> Result<Response, ContractError> {
        ensure!(
            self.whitelist.is_admin(ctx.deps.as_ref(), &ctx.info.sender),
            ContractError::Unauthorized
        );

        let spender = ctx.deps.api.addr_validate(&spender)?;
        ensure_ne!(ctx.info.sender, spender, ContractError::CannotSetOwnAccount);

        self.allowances
            .update(ctx.deps.storage, &spender, |allow| {
                let prev_expires = allow
                    .as_ref()
                    .map(|allow| allow.expires)
                    .unwrap_or_default();

                let mut allowance = allow
                    .filter(|allow| !allow.expires.is_expired(&ctx.env.block))
                    .unwrap_or_default();

                if let Some(exp) = expires {
                    if exp.is_expired(&ctx.env.block) {
                        return Err(ContractError::SettingExpiredAllowance(exp));
                    }

                    allowance.expires = exp;
                } else if prev_expires.is_expired(&ctx.env.block) {
                    return Err(ContractError::SettingExpiredAllowance(prev_expires));
                }

                allowance.balance += amount.clone();
                Ok(allowance)
            })?;

        let res = Response::new()
            .add_attribute("action", "increase_allowance")
            .add_attribute("owner", ctx.info.sender)
            .add_attribute("spender", spender)
            .add_attribute("denomination", amount.denom)
            .add_attribute("amount", amount.amount);
        Ok(res)
    }

    #[sv::msg(exec)]
    pub fn decrease_allowance(
        &self,
        ctx: ExecCtx,
        spender: String,
        amount: Coin,
        expires: Option<Expiration>,
    ) -> Result<Response, ContractError> {
        ensure!(
            self.whitelist.is_admin(ctx.deps.as_ref(), &ctx.info.sender),
            ContractError::Unauthorized
        );

        let spender = ctx.deps.api.addr_validate(&spender)?;
        ensure_ne!(ctx.info.sender, spender, ContractError::CannotSetOwnAccount);

        let allowance = self
            .allowances
            .update(ctx.deps.storage, &spender, |allow| {
                // Fail fast
                let mut allowance = allow
                    .filter(|allow| !allow.expires.is_expired(&ctx.env.block))
                    .ok_or(ContractError::NoAllowance)?;

                if let Some(exp) = expires {
                    if exp.is_expired(&ctx.env.block) {
                        return Err(ContractError::SettingExpiredAllowance(exp));
                    }

                    allowance.expires = exp;
                }

                // Tolerates underflows (amount bigger than balance), but fails if there are no tokens at all for the denom (report potential errors)
                allowance.balance = allowance.balance.sub_saturating(amount.clone())?;

                Ok(allowance)
            })?;

        if allowance.balance.is_empty() {
            self.allowances.remove(ctx.deps.storage, &spender);
        }

        let res = Response::new()
            .add_attribute("action", "decrease_allowance")
            .add_attribute("owner", ctx.info.sender)
            .add_attribute("spender", spender)
            .add_attribute("denomination", amount.denom)
            .add_attribute("amount", amount.amount);
        Ok(res)
    }

    #[sv::msg(exec)]
    pub fn set_permissions(
        &self,
        ctx: ExecCtx,
        spender: String,
        permissions: Permissions,
    ) -> Result<Response, ContractError> {
        ensure!(
            self.whitelist.is_admin(ctx.deps.as_ref(), &ctx.info.sender),
            ContractError::Unauthorized
        );

        let spender = ctx.deps.api.addr_validate(&spender)?;
        ensure_ne!(ctx.info.sender, spender, ContractError::CannotSetOwnAccount);
        self.permissions
            .save(ctx.deps.storage, &spender, &permissions)?;

        let res = Response::new()
            .add_attribute("action", "set_permissions")
            .add_attribute("owner", ctx.info.sender)
            .add_attribute("spender", spender)
            .add_attribute("permissions", permissions.to_string());
        Ok(res)
    }

    #[sv::msg(query)]
    pub fn allowance(&self, ctx: QueryCtx, spender: String) -> StdResult<Allowance> {
        // we can use unchecked here as it is a query - bad value means a miss, we never write it
        let spender = Addr::unchecked(spender);
        let allow = self
            .allowances
            .may_load(ctx.deps.storage, &spender)?
            .filter(|allow| !allow.expires.is_expired(&ctx.env.block))
            .unwrap_or_default();

        Ok(allow)
    }

    #[sv::msg(query)]
    pub fn permissions(&self, ctx: QueryCtx, spender: String) -> StdResult<Permissions> {
        let spender = Addr::unchecked(spender);
        let permissions = self
            .permissions
            .may_load(ctx.deps.storage, &spender)?
            .unwrap_or_default();

        Ok(permissions)
    }

    #[sv::msg(query)]
    pub fn all_allowances(
        &self,
        ctx: QueryCtx,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<AllAllowancesResponse> {
        let limit = calc_limit(limit);
        // we use raw addresses here....
        let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

        let allowances: StdResult<_> = self
            .allowances
            .range(ctx.deps.storage, start, None, Order::Ascending)
            .filter(|item| {
                if let Ok((_, allow)) = item {
                    !allow.expires.is_expired(&ctx.env.block)
                } else {
                    true
                }
            })
            .take(limit)
            .map(|item| {
                item.map(|(addr, allow)| AllowanceInfo {
                    spender: addr,
                    balance: allow.balance,
                    expires: allow.expires,
                })
            })
            .collect();

        Ok(AllAllowancesResponse {
            allowances: allowances?,
        })
    }

    #[sv::msg(query)]
    pub fn all_permissions(
        &self,
        ctx: QueryCtx,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<AllPermissionsResponse> {
        let limit = calc_limit(limit);
        let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

        let permissions: StdResult<_> = self
            .permissions
            .range(ctx.deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| {
                item.map(|(addr, perm)| PermissionsInfo {
                    spender: addr,
                    permissions: perm,
                })
            })
            .collect();

        Ok(AllPermissionsResponse {
            permissions: permissions?,
        })
    }

    pub fn is_authorized(
        &self,
        deps: Deps,
        env: &Env,
        sender: &Addr,
        msg: &CosmosMsg,
    ) -> StdResult<bool> {
        if self.whitelist.is_admin(deps, sender) {
            return Ok(true);
        }

        match msg {
            CosmosMsg::Bank(BankMsg::Send { amount, .. }) => {
                // now we check if there is enough allowance for this message
                let allowance = self.allowances.may_load(deps.storage, sender)?;
                match allowance {
                    // if there is an allowance, we subtract the requested amount to ensure it is covered (error on underflow)
                    Some(allow) => Ok(!allow.expires.is_expired(&env.block)
                        && (allow.balance - amount.clone()).is_ok()),
                    None => Ok(false),
                }
            }
            CosmosMsg::Staking(staking_msg) => {
                let permissions = match self.permissions.may_load(deps.storage, sender)? {
                    Some(permissions) => permissions,
                    None => return Ok(false),
                };

                let delegate =
                    matches!(staking_msg, StakingMsg::Delegate { .. } if permissions.delegate);
                let undelegate =
                    matches!(staking_msg, StakingMsg::Undelegate { .. } if permissions.undelegate);
                let redelegate =
                    matches!(staking_msg, StakingMsg::Redelegate { .. } if permissions.redelegate);

                Ok(delegate || undelegate || redelegate)
            }
            CosmosMsg::Distribution(distribution_msg) => {
                let permissions = match self.permissions.may_load(deps.storage, sender)? {
                    Some(permissions) => permissions,
                    None => return Ok(false),
                };

                let set_withdraw_addr = matches!(distribution_msg, DistributionMsg::SetWithdrawAddress { .. } if permissions.withdraw);
                let withdraw_perm = matches!(distribution_msg, DistributionMsg::WithdrawDelegatorReward { .. } if permissions.withdraw);

                Ok(set_withdraw_addr || withdraw_perm)
            }
            _ => Ok(false),
        }
    }
}

fn calc_limit(request: Option<u32>) -> usize {
    request.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize
}

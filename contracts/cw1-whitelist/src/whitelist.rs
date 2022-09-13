use cosmwasm_std::{Deps, DepsMut, Empty, Env, MessageInfo, Order, Response, StdResult};
use sylvia::interface;

use crate::contract::Cw1WhitelistContract;
use crate::error::ContractError;
use crate::responses::AdminListResponse;

#[interface]
pub trait Whitelist {
    type Error: From<cosmwasm_std::StdError>;

    #[msg(exec)]
    fn freeze(&self, ctx: (DepsMut, Env, MessageInfo)) -> Result<Response, Self::Error>;

    #[msg(exec)]
    fn update_admins(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        admins: Vec<String>,
    ) -> Result<Response, Self::Error>;

    #[msg(query)]
    fn admin_list(&self, ctx: (Deps, Env)) -> StdResult<AdminListResponse>;
}

impl Whitelist for Cw1WhitelistContract<'_> {
    type Error = ContractError;

    fn freeze(&self, ctx: (DepsMut, Env, MessageInfo)) -> Result<Response, ContractError> {
        let (deps, _, info) = ctx;

        if !self.is_admin(deps.as_ref(), &info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        self.mutable.save(deps.storage, &false)?;

        let resp = Response::new().add_attribute("action", "freeze");
        Ok(resp)
    }

    fn update_admins(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        mut admins: Vec<String>,
    ) -> Result<Response, ContractError> {
        let (deps, _, info) = ctx;

        if !self.is_admin(deps.as_ref(), &info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        if !self.mutable.load(deps.storage)? {
            return Err(ContractError::ContractFrozen {});
        }

        admins.sort_unstable();
        let mut low_idx = 0;

        let to_remove: Vec<_> = self
            .admins
            .keys(deps.storage, None, None, Order::Ascending)
            .filter(|addr| {
                // This is a bit of optimization basing on the fact that both `admins` and queried
                // keys range are sorted. Binary search would always return the index which is at
                // most as big as searched item, so for next item there is no point in looking at
                // lower indices. On the other hand - if we reached and of the sequence, we want to
                // remove all following keys.
                addr.as_ref()
                    .map(|addr| {
                        if low_idx >= admins.len() {
                            return true;
                        }

                        match admins[low_idx..].binary_search(&addr.into()) {
                            Ok(idx) => {
                                low_idx = idx;
                                false
                            }
                            Err(idx) => {
                                low_idx = idx;
                                true
                            }
                        }
                    })
                    .unwrap_or(true)
            })
            .collect::<Result<_, _>>()?;

        for addr in to_remove {
            self.admins.remove(deps.storage, &addr);
        }

        for admin in admins {
            let admin = deps.api.addr_validate(&admin)?;
            self.admins.save(deps.storage, &admin, &Empty {})?;
        }

        let resp = Response::new().add_attribute("action", "update_admins");
        Ok(resp)
    }

    fn admin_list(&self, ctx: (Deps, Env)) -> StdResult<AdminListResponse> {
        let (deps, _) = ctx;

        let admins: Result<_, _> = self
            .admins
            .keys(deps.storage, None, None, Order::Ascending)
            .map(|addr| addr.map(String::from))
            .collect();

        Ok(AdminListResponse {
            admins: admins?,
            mutable: self.mutable.load(deps.storage)?,
        })
    }
}

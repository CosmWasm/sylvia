use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw1_whitelist::responses::AdminListResponse;
#[cfg(test)]
use cw1_whitelist::whitelist;
use cw1_whitelist::whitelist::Whitelist;
use sylvia::contract;

use crate::contract::Cw1SubkeysContract;
use crate::error::ContractError;

// This can be skipped by tarpaulin as it's covered in cw1-whitelist
#[cfg(not(tarpaulin_include))]
#[contract]
#[messages(whitelist as Whitelist)]
impl Whitelist for Cw1SubkeysContract<'_> {
    type Error = ContractError;

    #[msg(exec)]
    fn freeze(&self, ctx: (DepsMut, Env, MessageInfo)) -> Result<Response, Self::Error> {
        self.whitelist.freeze(ctx).map_err(From::from)
    }

    #[msg(exec)]
    fn update_admins(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        admins: Vec<String>,
    ) -> Result<Response, Self::Error> {
        self.whitelist
            .update_admins(ctx, admins)
            .map_err(From::from)
    }

    #[msg(query)]
    fn admin_list(&self, ctx: (Deps, Env)) -> StdResult<AdminListResponse> {
        self.whitelist.admin_list(ctx)
    }
}

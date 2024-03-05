use cosmwasm_std::{Response, StdResult};
use sylvia::contract;
use sylvia::types::{ExecCtx, QueryCtx};
use whitelist::responses::AdminListResponse;
use whitelist::Whitelist;

use crate::contract::Cw1SubkeysContract;
use crate::error::ContractError;

// This can be skipped by tarpaulin as it's covered in cw1-whitelist
#[cfg(not(tarpaulin_include))]
#[contract(module=crate::contract)]
#[sv::messages(whitelist as Whitelist)]
impl Whitelist for Cw1SubkeysContract<'_> {
    type Error = ContractError;

    #[sv::msg(exec)]
    fn freeze(&self, ctx: ExecCtx) -> Result<Response, Self::Error> {
        self.whitelist.freeze(ctx).map_err(From::from)
    }

    #[sv::msg(exec)]
    fn update_admins(&self, ctx: ExecCtx, admins: Vec<String>) -> Result<Response, Self::Error> {
        self.whitelist
            .update_admins(ctx, admins)
            .map_err(From::from)
    }

    #[sv::msg(query)]
    fn admin_list(&self, ctx: QueryCtx) -> StdResult<AdminListResponse> {
        self.whitelist.admin_list(ctx)
    }
}

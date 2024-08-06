use cosmwasm_std::{Empty, Response, StdResult};
use sylvia::types::{ExecCtx, QueryCtx};
use whitelist::responses::AdminListResponse;
use whitelist::Whitelist;

use crate::contract::Cw1SubkeysContract;
use crate::error::ContractError;

impl Whitelist for Cw1SubkeysContract {
    type Error = ContractError;
    type ExecC = Empty;
    type QueryC = Empty;

    fn freeze(&self, ctx: ExecCtx) -> Result<Response, Self::Error> {
        self.whitelist.freeze(ctx).map_err(From::from)
    }

    fn update_admins(&self, ctx: ExecCtx, admins: Vec<String>) -> Result<Response, Self::Error> {
        self.whitelist
            .update_admins(ctx, admins)
            .map_err(From::from)
    }

    fn admin_list(&self, ctx: QueryCtx) -> StdResult<AdminListResponse> {
        self.whitelist.admin_list(ctx)
    }
}

use cosmwasm_std::{Response, StdResult};
use sylvia::types::{ExecCtx, QueryCtx};
use whitelist::responses::AdminListResponse;
use whitelist::Whitelist;

use crate::contract::Cw1SubkeysContract;
use crate::error::ContractError;
use cosmwasm_std::ensure;

impl Whitelist for Cw1SubkeysContract<'_> {
    type Error = ContractError;

    fn execute2(
        &self,
        ctx: ExecCtx,
        msgs: Vec<cosmwasm_std::CosmosMsg>,
    ) -> Result<cosmwasm_std::Response, Self::Error> {
        let authorized: StdResult<_> = msgs.iter().try_fold(true, |acc, msg| {
            Ok(acc & self.is_authorized(ctx.deps.as_ref(), &ctx.env, &ctx.info.sender, msg)?)
        });

        ensure!(authorized?, ContractError::Unauthorized);

        let res = Response::new()
            .add_messages(msgs)
            .add_attribute("action", "execute")
            .add_attribute("owner", ctx.info.sender);
        Ok(res)
    }

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

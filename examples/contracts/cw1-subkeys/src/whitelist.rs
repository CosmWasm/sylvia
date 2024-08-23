use sylvia::cw_std::{Response, StdResult};
use sylvia::types::{CustomMsg, CustomQuery, ExecCtx, QueryCtx};
use whitelist::responses::AdminListResponse;
use whitelist::Whitelist;

use crate::contract::Cw1SubkeysContract;
use crate::error::ContractError;

impl<E, Q> Whitelist for Cw1SubkeysContract<E, Q>
where
    E: CustomMsg + 'static,
    Q: CustomQuery + 'static,
{
    type Error = ContractError;
    type ExecC = E;
    type QueryC = Q;

    fn freeze(&self, ctx: ExecCtx<Self::QueryC>) -> Result<Response<Self::ExecC>, Self::Error> {
        self.whitelist.freeze(ctx).map_err(From::from)
    }

    fn update_admins(
        &self,
        ctx: ExecCtx<Self::QueryC>,
        admins: Vec<String>,
    ) -> Result<Response<Self::ExecC>, Self::Error> {
        self.whitelist
            .update_admins(ctx, admins)
            .map_err(From::from)
    }

    fn admin_list(&self, ctx: QueryCtx<Self::QueryC>) -> StdResult<AdminListResponse> {
        self.whitelist.admin_list(ctx)
    }
}

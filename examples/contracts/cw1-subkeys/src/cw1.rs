use cw1::{CanExecuteResp, Cw1};
use sylvia::cw_std::{ensure, Addr, CosmosMsg, Response, StdResult};
use sylvia::types::{CustomMsg, CustomQuery, ExecCtx, QueryCtx};

use crate::contract::Cw1SubkeysContract;
use crate::error::ContractError;

impl<E, Q> Cw1 for Cw1SubkeysContract<E, Q>
where
    E: CustomMsg + 'static,
    Q: CustomQuery + 'static,
{
    type Error = ContractError;
    type ExecC = E;
    type QueryC = Q;

    fn execute(
        &self,
        ctx: ExecCtx<Self::QueryC>,
        msgs: Vec<CosmosMsg<Self::ExecC>>,
    ) -> Result<Response<Self::ExecC>, Self::Error> {
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

    fn can_execute(
        &self,
        ctx: QueryCtx<Self::QueryC>,
        sender: String,
        msg: CosmosMsg<Self::ExecC>,
    ) -> StdResult<CanExecuteResp> {
        let sender = Addr::unchecked(sender);

        self.is_authorized(ctx.deps, &ctx.env, &sender, &msg)
            .map(|can| CanExecuteResp { can_execute: can })
    }
}

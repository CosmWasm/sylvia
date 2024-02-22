use cosmwasm_std::{ensure, Addr, Response, StdResult};
use cw1::{CanExecuteResp, Cw1};
use sylvia::contract;
use sylvia::types::{ExecCtx, QueryCtx};

use crate::contract::Cw1SubkeysContract;
use crate::error::ContractError;

#[contract(module=crate::contract)]
#[sv::messages(cw1 as Cw1)]
impl Cw1 for Cw1SubkeysContract<'_> {
    type Error = ContractError;

    #[sv::msg(exec)]
    fn execute(
        &self,
        ctx: ExecCtx,
        msgs: Vec<cosmwasm_std::CosmosMsg>,
    ) -> Result<cosmwasm_std::Response, Self::Error> {
        let authorized: StdResult<_> = msgs.iter().fold(Ok(true), |acc, msg| {
            Ok(acc? & self.is_authorized(ctx.deps.as_ref(), &ctx.env, &ctx.info.sender, msg)?)
        });

        ensure!(authorized?, ContractError::Unauthorized);

        let res = Response::new()
            .add_messages(msgs)
            .add_attribute("action", "execute")
            .add_attribute("owner", ctx.info.sender);
        Ok(res)
    }

    #[sv::msg(query)]
    fn can_execute(
        &self,
        ctx: QueryCtx,
        sender: String,
        msg: cosmwasm_std::CosmosMsg,
    ) -> StdResult<CanExecuteResp> {
        let sender = Addr::unchecked(sender);

        self.is_authorized(ctx.deps, &ctx.env, &sender, &msg)
            .map(|can| CanExecuteResp { can_execute: can })
    }
}

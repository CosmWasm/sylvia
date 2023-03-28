use cosmwasm_std::{Addr, CosmosMsg, Response, StdResult};
use cw1::{CanExecuteResp, Cw1};
use sylvia::contract;
use sylvia::types::{ExecCtx, QueryCtx};

use crate::contract::Cw1WhitelistContract;
use crate::error::ContractError;

#[contract]
#[messages(cw1 as Cw1)]
impl Cw1 for Cw1WhitelistContract<'_> {
    type Error = ContractError;

    #[msg(exec)]
    fn execute(&self, ctx: ExecCtx, msgs: Vec<CosmosMsg>) -> Result<Response, ContractError> {
        if !self.is_admin(ctx.deps.as_ref(), &ctx.info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        let resp = Response::new()
            .add_messages(msgs)
            .add_attribute("action", "execute");
        Ok(resp)
    }

    #[msg(query)]
    fn can_execute(
        &self,
        ctx: QueryCtx,
        sender: String,
        _msg: CosmosMsg,
    ) -> StdResult<CanExecuteResp> {
        let resp = CanExecuteResp {
            can_execute: self.is_admin(ctx.deps, &Addr::unchecked(sender)),
        };

        Ok(resp)
    }
}

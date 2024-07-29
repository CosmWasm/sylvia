use cosmwasm_std::{Addr, CosmosMsg, Response, StdResult};
use cw1::{CanExecuteResp, Cw1};
use sylvia::types::{CustomMsg, CustomQuery, ExecCtx, QueryCtx};

use crate::contract::Cw1WhitelistContract;
use crate::error::ContractError;

impl<E, Q> Cw1 for Cw1WhitelistContract<E, Q>
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
    ) -> Result<Response<Self::ExecC>, ContractError> {
        if !self.is_admin(ctx.deps.as_ref(), &ctx.info.sender) {
            return Err(ContractError::Unauthorized);
        }

        let resp = Response::new()
            .add_messages(msgs)
            .add_attribute("action", "execute");
        Ok(resp)
    }

    fn can_execute(
        &self,
        ctx: QueryCtx<Self::QueryC>,
        sender: String,
        _msg: CosmosMsg<Self::ExecC>,
    ) -> StdResult<CanExecuteResp> {
        let resp = CanExecuteResp {
            can_execute: self.is_admin(ctx.deps, &Addr::unchecked(sender)),
        };

        Ok(resp)
    }
}

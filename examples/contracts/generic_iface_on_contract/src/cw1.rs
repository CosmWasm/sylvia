use cosmwasm_std::{CosmosMsg, Response, StdError, StdResult};
use cw1::{CanExecuteResp, Cw1};
use sylvia::types::{ExecCtx, QueryCtx};

impl Cw1 for crate::contract::NonGenericContract {
    type Error = StdError;

    fn execute(&self, _ctx: ExecCtx, _msgs: Vec<CosmosMsg>) -> StdResult<Response> {
        Ok(Response::new())
    }

    fn can_execute(
        &self,
        _ctx: QueryCtx,
        _sender: String,
        _msg: CosmosMsg,
    ) -> StdResult<CanExecuteResp> {
        Ok(CanExecuteResp::default())
    }
}

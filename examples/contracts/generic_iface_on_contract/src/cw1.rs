use cw1::{CanExecuteResp, Cw1};
use sylvia::ctx::{ExecCtx, QueryCtx};
use sylvia::cw_std::{CosmosMsg, Empty, Response, StdError, StdResult};

impl Cw1 for crate::contract::NonGenericContract {
    type Error = StdError;
    type ExecC = Empty;
    type QueryC = Empty;

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

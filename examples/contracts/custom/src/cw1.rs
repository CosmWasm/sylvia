use cosmwasm_std::{CosmosMsg, Empty, Response, StdError, StdResult};
use cw1::{CanExecuteResp, Cw1};
use sylvia::types::{ExecCtx, QueryCtx};

use crate::contract::CustomContract;

impl Cw1 for CustomContract {
    type Error = StdError;
    type ExecC = Empty;
    type QueryC = Empty;
    type CosmosCustomMsg = Empty;

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

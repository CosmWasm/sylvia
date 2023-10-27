use crate::messages::CounterMsg;
use cosmwasm_std::{CosmosMsg, Response, StdError, StdResult};
use cw1::{CanExecuteResp, Cw1};
use sylvia::contract;
use sylvia::types::{ExecCtx, QueryCtx};

use crate::contract::CustomContract;

#[contract(module=crate::contract)]
#[messages(cw1 as Cw1)]
#[sv::custom(query=CounterQuery, msg=CounterMsg)]
impl Cw1 for CustomContract {
    type Error = StdError;

    #[msg(exec)]
    fn execute(&self, _ctx: ExecCtx, _msgs: Vec<CosmosMsg>) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[msg(query)]
    fn can_execute(
        &self,
        _ctx: QueryCtx,
        _sender: String,
        _msg: CosmosMsg,
    ) -> StdResult<CanExecuteResp> {
        Ok(CanExecuteResp::default())
    }
}

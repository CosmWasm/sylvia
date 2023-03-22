use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdResult};
use cw_multi_test::{App, Executor};
use sylvia::{contract, schemars};

use super::receiver;
pub struct ReceiverContract {}

#[contract]
#[messages(receiver as Receiver)]
impl ReceiverContract {
    pub const fn new() -> Self {
        Self {}
    }
    #[msg(instantiate)]
    pub fn instantiate(&self, _ctx: (DepsMut, Env, MessageInfo)) -> StdResult<Response> {
        Ok(Response::new())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReceiverContractCodeId(u64);

impl ReceiverContractCodeId {
    pub fn store_code(app: &mut App) -> Self {
        let code_id = app.store_code(Box::new(ReceiverContract::new()));
        Self(code_id)
    }

    #[track_caller]
    pub fn instantiate(
        self,
        app: &mut App,
        sender: &Addr,
        label: &str,
    ) -> StdResult<ReceiverContractProxy> {
        let msg = InstantiateMsg {};

        app.instantiate_contract(self.0, sender.clone(), &msg, &[], label, None)
            .map_err(|err| err.downcast().unwrap())
            .map(ReceiverContractProxy)
    }
}

pub struct ReceiverContractProxy(Addr);

impl ReceiverContractProxy {
    pub fn addr(&self) -> &Addr {
        &self.0
    }
}

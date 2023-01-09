use std::cell::RefCell;
use std::marker::PhantomData;

use cosmwasm_std::{Addr, Coin};
use cw_multi_test::Executor;
use serde::Serialize;

#[derive(Default)]
pub struct App {
    pub app: RefCell<cw_multi_test::App>,
}

#[must_use]
pub struct ExecProxy<'a, 'app, Error, Msg>
where
    Msg: Serialize + std::fmt::Debug,
    Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
{
    funds: &'a [Coin],
    sender: &'a str,
    contract_addr: &'a Addr,
    msg: Msg,
    app: &'app App,
    phantom: PhantomData<Error>,
}

impl<'a, 'app, Error, Msg> ExecProxy<'a, 'app, Error, Msg>
where
    Msg: Serialize + std::fmt::Debug,
    Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
{
    pub fn new(contract_addr: &'a Addr, msg: Msg, app: &'app App) -> Self {
        Self {
            funds: &[],
            sender: "",
            contract_addr,
            msg,
            app,
            phantom: PhantomData,
        }
    }
    pub fn with_funds(self, funds: &'a [Coin]) -> Self {
        Self { funds, ..self }
    }

    pub fn with_sender(self, sender: &'a str) -> Self {
        Self { sender, ..self }
    }

    #[track_caller]
    pub fn call(self) -> Result<cw_multi_test::AppResponse, Error> {
        self.app
            .app
            .borrow_mut()
            .execute_contract(
                Addr::unchecked(self.sender),
                Addr::unchecked(self.contract_addr),
                &self.msg,
                self.funds,
            )
            .map_err(|err| err.downcast().unwrap())
    }
}

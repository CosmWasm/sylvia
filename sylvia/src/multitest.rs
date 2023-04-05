use std::cell::{Ref, RefCell, RefMut};
use std::marker::PhantomData;

use cosmwasm_std::{Addr, BlockInfo, Coin};
use cw_multi_test::Executor;
use serde::Serialize;

#[derive(Default)]
pub struct App {
    pub app: RefCell<cw_multi_test::App>,
}

impl App {
    pub fn new(app: cw_multi_test::App) -> Self {
        Self {
            app: RefCell::new(app),
        }
    }

    pub fn app(&self) -> Ref<'_, cw_multi_test::App> {
        Ref::map(self.app.borrow(), |app| app)
    }

    pub fn app_mut(&self) -> RefMut<'_, cw_multi_test::App> {
        RefMut::map(self.app.borrow_mut(), |app| app)
    }

    pub fn block_info(&self) -> BlockInfo {
        self.app.borrow().block_info()
    }

    pub fn set_block(&self, block: BlockInfo) {
        self.app.borrow_mut().set_block(block)
    }

    pub fn update_block<F: Fn(&mut BlockInfo)>(&self, action: F) {
        self.app.borrow_mut().update_block(action)
    }
}

#[must_use]
pub struct ExecProxy<'a, 'app, Error, Msg>
where
    Msg: Serialize + std::fmt::Debug,
    Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
{
    funds: &'a [Coin],
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
            contract_addr,
            msg,
            app,
            phantom: PhantomData,
        }
    }
    pub fn with_funds(self, funds: &'a [Coin]) -> Self {
        Self { funds, ..self }
    }

    #[track_caller]
    pub fn call(self, sender: &'a str) -> Result<cw_multi_test::AppResponse, Error> {
        self.app
            .app
            .borrow_mut()
            .execute_contract(
                Addr::unchecked(sender),
                Addr::unchecked(self.contract_addr),
                &self.msg,
                self.funds,
            )
            .map_err(|err| err.downcast().unwrap())
    }
}

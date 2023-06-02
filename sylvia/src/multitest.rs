use std::cell::{Ref, RefCell, RefMut};
use std::fmt::Debug;
use std::marker::PhantomData;

use cosmwasm_std::{
    Addr, Api, BlockInfo, Coin, CustomQuery, Empty, GovMsg, IbcMsg, IbcQuery, Storage,
};
use cw_multi_test::{
    AppBuilder, BankKeeper, DistributionKeeper, Executor, FailingModule, Router, StakeKeeper,
    WasmKeeper,
};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct App<ExecC = Empty, QueryC = Empty> {
    pub app: RefCell<cw_multi_test::BasicApp<ExecC, QueryC>>,
}

impl Default for App {
    fn default() -> Self {
        Self::new(cw_multi_test::BasicApp::default())
    }
}

/// Creates new default `App` implementation working with customized exec and query messages.
/// Outside of `App` implementation to make type elision better.
pub fn custom_app<ExecC, QueryC, F>(init_fn: F) -> App<ExecC, QueryC>
where
    ExecC: Debug + Clone + PartialEq + JsonSchema + DeserializeOwned + 'static,
    QueryC: Debug + CustomQuery + DeserializeOwned + 'static,
    F: FnOnce(
        &mut Router<
            BankKeeper,
            FailingModule<ExecC, QueryC, Empty>,
            WasmKeeper<ExecC, QueryC>,
            StakeKeeper,
            DistributionKeeper,
            FailingModule<IbcMsg, IbcQuery, Empty>,
            FailingModule<GovMsg, Empty, Empty>,
        >,
        &dyn Api,
        &mut dyn Storage,
    ),
{
    App {
        app: RefCell::new(AppBuilder::new_custom().build(init_fn)),
    }
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
pub struct ExecProxy<'a, 'app, Error, Msg, ExecC = Empty, QueryC = Empty>
where
    Msg: Serialize + std::fmt::Debug,
    Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    ExecC: Debug + Clone + PartialEq + JsonSchema + DeserializeOwned + 'static,
    QueryC: Debug + CustomQuery + DeserializeOwned + 'static,
{
    funds: &'a [Coin],
    contract_addr: &'a Addr,
    msg: Msg,
    app: &'app App<ExecC, QueryC>,
    phantom: PhantomData<Error>,
}

impl<'a, 'app, Error, Msg, ExecC, QueryC> ExecProxy<'a, 'app, Error, Msg, ExecC, QueryC>
where
    Msg: Serialize + std::fmt::Debug,
    Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    ExecC: Debug + Clone + PartialEq + JsonSchema + DeserializeOwned + 'static,
    QueryC: Debug + CustomQuery + DeserializeOwned + 'static,
{
    pub fn new(contract_addr: &'a Addr, msg: Msg, app: &'app App<ExecC, QueryC>) -> Self {
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

#[must_use]
pub struct MigrateProxy<'a, 'app, Error, Msg, ExecC = Empty, QueryC = Empty>
where
    Msg: Serialize + std::fmt::Debug,
    Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    ExecC: Debug + Clone + PartialEq + JsonSchema + DeserializeOwned + 'static,
    QueryC: Debug + CustomQuery + DeserializeOwned + 'static,
{
    contract_addr: &'a Addr,
    msg: Msg,
    app: &'app App<ExecC, QueryC>,
    phantom: PhantomData<Error>,
}

impl<'a, 'app, Error, Msg, ExecC, QueryC> MigrateProxy<'a, 'app, Error, Msg, ExecC, QueryC>
where
    Msg: Serialize + std::fmt::Debug,
    Error: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    ExecC: Debug + Clone + PartialEq + JsonSchema + DeserializeOwned + 'static,
    QueryC: Debug + CustomQuery + DeserializeOwned + 'static,
{
    pub fn new(contract_addr: &'a Addr, msg: Msg, app: &'app App<ExecC, QueryC>) -> Self {
        Self {
            contract_addr,
            msg,
            app,
            phantom: PhantomData,
        }
    }

    #[track_caller]
    pub fn call(self, sender: &str, new_code_id: u64) -> Result<cw_multi_test::AppResponse, Error> {
        self.app
            .app
            .borrow_mut()
            .migrate_contract(
                Addr::unchecked(sender),
                Addr::unchecked(self.contract_addr),
                &self.msg,
                new_code_id,
            )
            .map_err(|err| err.downcast().unwrap())
    }
}

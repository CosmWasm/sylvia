use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::ops::Deref;

use cosmwasm_std::{Addr, Api, Coin, CustomQuery, Empty, GovMsg, IbcMsg, IbcQuery, Storage};
use cw_multi_test::{
    BankKeeper, DistributionKeeper, Executor, FailingModule, Router, StakeKeeper, WasmKeeper,
};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct App<MtApp> {
    app: RefCell<MtApp>,
}

impl<MtApp> Default for App<MtApp>
where
    MtApp: Default,
{
    fn default() -> Self {
        Self::new(MtApp::default())
    }
}

impl<MtApp> Deref for App<MtApp> {
    type Target = RefCell<MtApp>;

    fn deref(&self) -> &Self::Target {
        &self.app
    }
}

impl<ExecC, QueryC> App<cw_multi_test::BasicApp<ExecC, QueryC>> {
    /// Creates new default `App` implementation working with customized exec and query messages.
    pub fn custom<F>(init_fn: F) -> Self
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
            app: RefCell::new(cw_multi_test::custom_app(init_fn)),
        }
    }
}

impl<MtApp> App<MtApp> {
    pub fn new(app: MtApp) -> Self {
        Self {
            app: RefCell::new(app),
        }
    }

    pub fn app(&self) -> Ref<'_, MtApp> {
        Ref::map(self.app.borrow(), |app| app)
    }

    pub fn app_mut(&self) -> RefMut<'_, MtApp> {
        RefMut::map(self.app.borrow_mut(), |app| app)
    }
}

#[must_use]
pub struct ExecProxy<'a, 'app, Error, Msg, MtApp, ExecC = Empty>
where
    Msg: Serialize + Debug,
    Error: Debug + Display + Send + Sync + 'static,
{
    funds: &'a [Coin],
    contract_addr: &'a Addr,
    msg: Msg,
    app: &'app App<MtApp>,
    phantom: PhantomData<(Error, ExecC)>,
}

impl<'a, 'app, Error, Msg, MtApp, ExecC> ExecProxy<'a, 'app, Error, Msg, MtApp, ExecC>
where
    Msg: Serialize + Debug,
    Error: Debug + Display + Send + Sync + 'static,
    ExecC: Debug + Clone + JsonSchema + PartialEq + 'static,
    MtApp: Executor<ExecC>,
{
    pub fn new(contract_addr: &'a Addr, msg: Msg, app: &'app App<MtApp>) -> Self {
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
        (*self.app)
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
pub struct MigrateProxy<'a, 'app, Error, Msg, MtApp, ExecC = Empty>
where
    Msg: Serialize + Debug,
    Error: Debug + Display + Send + Sync + 'static,
{
    contract_addr: &'a Addr,
    msg: Msg,
    app: &'app App<MtApp>,
    phantom: PhantomData<(Error, ExecC)>,
}

impl<'a, 'app, Error, Msg, MtApp, ExecC> MigrateProxy<'a, 'app, Error, Msg, MtApp, ExecC>
where
    Msg: Serialize + Debug,
    Error: Debug + Display + Send + Sync + 'static,
    ExecC: Debug + Clone + JsonSchema + PartialEq + 'static,
    MtApp: Executor<ExecC>,
{
    pub fn new(contract_addr: &'a Addr, msg: Msg, app: &'app App<MtApp>) -> Self {
        Self {
            contract_addr,
            msg,
            app,
            phantom: PhantomData,
        }
    }

    #[track_caller]
    pub fn call(self, sender: &str, new_code_id: u64) -> Result<cw_multi_test::AppResponse, Error> {
        (*self.app)
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

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, CustomMsg, CustomQuery, Empty, StdError};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
    struct MyMsg;

    impl CustomMsg for MyMsg {}

    #[derive(Clone, PartialEq, Serialize, Deserialize, Debug, JsonSchema)]
    struct MyQuery;

    impl CustomQuery for MyQuery {}

    #[test]
    fn construct_types() {
        // App
        let _ = super::App::<cw_multi_test::App>::default();
        let basic_app = super::App::new(cw_multi_test::BasicApp::default());
        let custom_app =
            super::App::<cw_multi_test::BasicApp<MyMsg, MyQuery>>::custom(|_, _, _| {});

        // ExecProxy
        let _: super::ExecProxy<StdError, Empty, cw_multi_test::BasicApp> =
            super::ExecProxy::new(&Addr::unchecked("addr"), Empty {}, &basic_app);
        let _: super::ExecProxy<StdError, Empty, cw_multi_test::BasicApp<MyMsg, MyQuery>, MyMsg> =
            super::ExecProxy::new(&Addr::unchecked("addr"), Empty {}, &custom_app);
    }
}

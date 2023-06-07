use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{Debug, Display};
use std::marker::PhantomData;

use cosmwasm_std::{
    Addr, Api, BlockInfo, Coin, CustomQuery, Empty, GovMsg, IbcMsg, IbcQuery, Storage,
};
use cw_multi_test::{
    Bank, BankKeeper, Distribution, DistributionKeeper, Executor, FailingModule, Gov, Ibc, Module,
    Router, StakeKeeper, Staking, Wasm, WasmKeeper,
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

impl<BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT>
    App<cw_multi_test::App<BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT>>
where
    CustomT::ExecT: std::fmt::Debug + PartialEq + Clone + JsonSchema + DeserializeOwned + 'static,
    CustomT::QueryT: CustomQuery + DeserializeOwned + 'static,
    WasmT: Wasm<CustomT::ExecT, CustomT::QueryT>,
    BankT: Bank,
    ApiT: Api,
    StorageT: Storage,
    CustomT: Module,
    StakingT: Staking,
    DistrT: Distribution,
    IbcT: Ibc,
    GovT: Gov,
{
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
pub struct ExecProxy<'a, 'app, Error, Msg, MtApp, ExecC>
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
            .app_mut()
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
pub struct MigrateProxy<'a, 'app, Error, Msg, MtApp, ExecC>
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
            .app_mut()
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
    use std::cell::{Ref, RefMut};

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

        let _: Ref<cw_multi_test::BasicApp> = basic_app.app();
        let _: RefMut<cw_multi_test::BasicApp> = basic_app.app_mut();

        // ExecProxy
        let _: super::ExecProxy<StdError, Empty, cw_multi_test::BasicApp, Empty> =
            super::ExecProxy::new(&Addr::unchecked("addr"), Empty {}, &basic_app);
        let _: super::ExecProxy<StdError, Empty, cw_multi_test::BasicApp<MyMsg, MyQuery>, MyMsg> =
            super::ExecProxy::new(&Addr::unchecked("addr"), Empty {}, &custom_app);

        // MigrateProxy
        let _: super::MigrateProxy<StdError, Empty, cw_multi_test::BasicApp, Empty> =
            super::MigrateProxy::new(&Addr::unchecked("addr"), Empty {}, &basic_app);
        let _: super::MigrateProxy<
            StdError,
            Empty,
            cw_multi_test::BasicApp<MyMsg, MyQuery>,
            MyMsg,
        > = super::MigrateProxy::new(&Addr::unchecked("addr"), Empty {}, &custom_app);
    }
}

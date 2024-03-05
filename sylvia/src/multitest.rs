//! This module provides utilities to work with `cw_multi_test` crate.
//!
//! ## Example usage:
//! ```rust
//! # use sylvia::cw_std::{Response, StdResult};
//! # use sylvia::types::{ExecCtx, InstantiateCtx, QueryCtx};
//! pub struct SvContract;
//!
//! ##[sylvia::contract]
//! impl SvContract {
//! #    pub const fn new() -> Self { Self }
//! #
//!     #[sv::msg(instantiate)]
//!     pub fn instantiate(&self, ctx: InstantiateCtx) -> StdResult<Response> {
//! #        Ok(Response::new())
//!     }
//!
//!     #[sv::msg(exec)]
//!     pub fn execute(&self, ctx: ExecCtx, param: u32) -> StdResult<Response> {
//! #        Ok(Response::new())
//!     }
//!
//!     #[sv::msg(query)]
//!     pub fn query(&self, ctx: QueryCtx) -> StdResult<Response> {
//! #        Ok(Response::new())
//!     }
//! }
//!
//! #[cfg(test)]
//! mod tests {
//! #   use super::*;
//!     #[test]
//!     fn example_test() {
//!         let app = sylvia::multitest::App::default();
//!         let code_id = sv::mt::CodeId::store_code(&app);
//!         let owner = "owner";
//!
//!         let contract = code_id
//!             .instantiate()
//!             .with_label("MyContract") // optional
//!             .with_admin(owner) // optional
//!             .call(owner)
//!             .unwrap();
//!
//!         contract.execute(42).call(owner).unwrap();
//!         contract.query().unwrap();
//!     }
//! }
//!
//! # fn main() {}
//! ```

use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{Debug, Display};
use std::marker::PhantomData;

use cosmwasm_std::{
    Addr, Api, BlockInfo, Coin, Empty, Querier, QuerierResult, QuerierWrapper, Storage,
};
#[cfg(feature = "cosmwasm_1_2")]
use cosmwasm_std::{CodeInfoResponse, StdResult};
use cw_multi_test::{
    Bank, BankKeeper, Distribution, DistributionKeeper, Executor, FailingModule, Gov,
    GovFailingModule, Ibc, IbcFailingModule, Module, Router, StakeKeeper, Staking, Stargate,
    StargateFailingModule, Wasm, WasmKeeper,
};
use derivative::Derivative;
use serde::Serialize;

use crate::types::{CustomMsg, CustomQuery};

/// Proxy to interact with a smart contract initialized on the [App].
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Proxy<'a, MtApp, Contract> {
    pub contract_addr: cosmwasm_std::Addr,
    #[derivative(Debug = "ignore")]
    pub app: &'a crate::multitest::App<MtApp>,
    pub _phantom: std::marker::PhantomData<(MtApp, Contract)>,
}

impl<'a, MtApp, Contract> Proxy<'a, MtApp, Contract> {
    pub fn new(contract_addr: cosmwasm_std::Addr, app: &'a App<MtApp>) -> Self {
        Proxy {
            contract_addr,
            app,
            _phantom: std::marker::PhantomData::<(MtApp, Contract)>,
        }
    }
}

impl<'app, MtApp, Contract> From<(cosmwasm_std::Addr, &'app App<MtApp>)>
    for Proxy<'app, MtApp, Contract>
{
    fn from(input: (cosmwasm_std::Addr, &'app App<MtApp>)) -> Self {
        Self::new(input.0, input.1)
    }
}

/// Wrapper around `cw_multi_test::App` to provide additional functionalities.
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
        ExecC: CustomMsg + 'static,
        QueryC: Debug + CustomQuery + 'static,
        F: FnOnce(
            &mut Router<
                BankKeeper,
                FailingModule<ExecC, QueryC, Empty>,
                WasmKeeper<ExecC, QueryC>,
                StakeKeeper,
                DistributionKeeper,
                IbcFailingModule,
                GovFailingModule,
                StargateFailingModule,
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

    /// Immutable borrow on the underlying `cw_multi_test::App`.
    pub fn app(&self) -> Ref<'_, MtApp> {
        Ref::map(self.app.borrow(), |app| app)
    }

    /// Mutable borrow on the underlying `cw_multi_test::App`.
    pub fn app_mut(&self) -> RefMut<'_, MtApp> {
        RefMut::map(self.app.borrow_mut(), |app| app)
    }
}

impl<BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT, StargateT>
    App<
        cw_multi_test::App<
            BankT,
            ApiT,
            StorageT,
            CustomT,
            WasmT,
            StakingT,
            DistrT,
            IbcT,
            GovT,
            StargateT,
        >,
    >
where
    CustomT::ExecT: CustomMsg + 'static,
    CustomT::QueryT: CustomQuery + 'static,
    WasmT: Wasm<CustomT::ExecT, CustomT::QueryT>,
    BankT: Bank,
    ApiT: Api,
    StorageT: Storage,
    CustomT: Module,
    StakingT: Staking,
    DistrT: Distribution,
    IbcT: Ibc,
    GovT: Gov,
    StargateT: Stargate,
{
    /// Returns the info of the current block on the chain.
    pub fn block_info(&self) -> BlockInfo {
        self.app.borrow().block_info()
    }

    /// Sets the info of the current block on the chain.
    pub fn set_block(&self, block: BlockInfo) {
        self.app.borrow_mut().set_block(block)
    }

    /// Updates the info of the current block on the chain.
    pub fn update_block<F: Fn(&mut BlockInfo)>(&self, action: F) {
        self.app.borrow_mut().update_block(action)
    }

    /// Returns [CodeInfoResponse] for the given `code_id`.
    #[cfg(feature = "cosmwasm_1_2")]
    pub fn code_info(&self, code_id: u64) -> StdResult<CodeInfoResponse> {
        self.querier().query_wasm_code_info(code_id)
    }

    /// Initialize a new [QuerierWrapper] used to call e.g. `query_wasm_smart` or
    /// `query_all_balances`.
    /// A counterpart to `cw_multi_test::App::wrap` method.
    pub fn querier(&self) -> QuerierWrapper<'_, CustomT::QueryT> {
        QuerierWrapper::new(self)
    }
}

impl<BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT, StargateT> Querier
    for App<
        cw_multi_test::App<
            BankT,
            ApiT,
            StorageT,
            CustomT,
            WasmT,
            StakingT,
            DistrT,
            IbcT,
            GovT,
            StargateT,
        >,
    >
where
    CustomT::ExecT: CustomMsg + 'static,
    CustomT::QueryT: CustomQuery + 'static,
    WasmT: Wasm<CustomT::ExecT, CustomT::QueryT>,
    BankT: Bank,
    ApiT: Api,
    StorageT: Storage,
    CustomT: Module,
    StakingT: Staking,
    DistrT: Distribution,
    IbcT: Ibc,
    GovT: Gov,
    StargateT: Stargate,
{
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        self.app.borrow().raw_query(bin_request)
    }
}

/// Intermiediate proxy to set additional information
/// before sending an execute message.
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
    ExecC: cosmwasm_std::CustomMsg + 'static,
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

    /// Sets the funds to be sent with the execute message.
    pub fn with_funds(self, funds: &'a [Coin]) -> Self {
        Self { funds, ..self }
    }

    /// Sends the execute message to the contract.
    #[track_caller]
    pub fn call(self, sender: &'a Addr) -> Result<cw_multi_test::AppResponse, Error> {
        (*self.app)
            .app_mut()
            .execute_contract(
                sender.clone(),
                Addr::unchecked(self.contract_addr),
                &self.msg,
                self.funds,
            )
            .map_err(|err| err.downcast().unwrap())
    }
}

/// Intermiediate proxy to set additional information
/// before sending an migrate message.
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
    ExecC: cosmwasm_std::CustomMsg + 'static,
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

    /// Sends the migrate message to the contract.
    #[track_caller]
    pub fn call(
        self,
        sender: &Addr,
        new_code_id: u64,
    ) -> Result<cw_multi_test::AppResponse, Error> {
        (*self.app)
            .app_mut()
            .migrate_contract(
                sender.clone(),
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

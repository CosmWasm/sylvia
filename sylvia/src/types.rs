//! Module providing utilities to build and use sylvia contracts.
use cosmwasm_std::{Binary, Coin, Deps, DepsMut, Empty, Env, MessageInfo, WasmMsg};
use derivative::Derivative;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Wrapper around [QuerierWrapper](cosmwasm_std::QuerierWrapper) for more user friendly query experience.
/// Most of the implementation should be provided via traits.
/// [contract](crate::contract) and [interface](crate::interface) macros will generate the required implementation
/// for each `query` message.
///
/// # Example
///
/// Call a query method on a remote contract.
///
/// ```rust
/// mod admin_contract {
/// #   use sylvia::types::{InstantiateCtx, QueryCtx};
/// #   use sylvia::cw_std::{Addr, Response, StdResult};
/// #
///     pub struct AdminContract;
///
///     #[sylvia::contract]
///     impl AdminContract {
/// #       pub const fn new() -> Self {
/// #           Self
/// #       }
/// #
/// #       #[sv::msg(instantiate)]
/// #       pub fn instantiate(&self, ctx: InstantiateCtx) -> StdResult<Response> {
/// #           Ok(Response::new())
/// #       }
/// #
///         #[sv::msg(query)]
///         pub fn is_admin(&self, ctx: QueryCtx, addr: Addr) -> StdResult<bool> {
///             /// Validate admin
/// #           Ok(true)
///         }
///     }
/// }
///
/// mod other_contract {
///     use sylvia::types::BoundQuerier;
///     use crate::admin_contract::sv::Querier;
/// #    use sylvia::cw_std::{Addr, Deps, StdResult};
/// #    use cw_storage_plus::Item;
///
///     pub const ADMIN_CONTRACT: Item<Addr> = Item::new("admin_contract");
///
///     fn is_admin(deps: Deps, addr: Addr) -> StdResult<bool> {
///         let admin_addr = ADMIN_CONTRACT.load(deps.storage)?;
///         BoundQuerier::borrowed(&admin_addr, &deps.querier)
///             .is_admin(addr)
///     }
/// }
///
/// fn main() {}
/// ```
///
/// You can also call a query method on a remote contract implementing some interface.
///
/// ```rust
/// mod admin_contract {
/// #   use sylvia::types::{InstantiateCtx, QueryCtx};
/// #   use sylvia::cw_std::{Addr, Response, StdResult};
/// #
///     pub struct AdminContract;
///
///     #[sylvia::contract]
///     #[sv::messages(crate::admin_interface as Admin)]
///     impl AdminContract {
/// #       pub const fn new() -> Self {
/// #           Self
/// #       }
/// #
/// #       #[sv::msg(instantiate)]
/// #       pub fn instantiate(&self, ctx: InstantiateCtx) -> StdResult<Response> {
/// #           Ok(Response::new())
/// #       }
/// #
///     }
/// }
///
/// mod admin_interface {
/// #   use sylvia::types::QueryCtx;
/// #   use sylvia::cw_std::{Addr, StdError, StdResult};
/// #   use crate::admin_contract::AdminContract;
/// #
///     #[sylvia::interface]
///     pub trait Admin {
/// #       type Error: From<StdError>;
/// #
///         #[sv::msg(query)]
///         fn is_admin(&self, ctx: QueryCtx, addr: Addr) -> StdResult<bool>;
///     }
///
///     impl Admin for AdminContract {
/// #       type Error = StdError;
///         fn is_admin(&self, ctx: QueryCtx, addr: Addr) -> StdResult<bool> {
///             /// Validate admin
/// #           Ok(true)
///         }
///     }
/// }
///
/// mod other_contract {
///     use sylvia::types::BoundQuerier;
///     use crate::admin_interface::sv::Querier;
///     use crate::admin_contract::AdminContract;
/// #    use sylvia::cw_std::{Addr, Deps, StdResult};
/// #    use cw_storage_plus::Item;
///
///     pub const ADMIN_CONTRACT: Item<Addr> = Item::new("admin_contract");
///
///     fn is_admin(deps: Deps, addr: Addr) -> StdResult<bool> {
///         let admin_addr = ADMIN_CONTRACT.load(deps.storage)?;
///         BoundQuerier::<_, AdminContract>::borrowed(&admin_addr, &deps.querier)
///             .is_admin(addr)
///     }
/// }
///
/// fn main() {}
/// ```
pub struct BoundQuerier<'a, C: cosmwasm_std::CustomQuery, Contract: ?Sized> {
    contract: &'a cosmwasm_std::Addr,
    querier: &'a cosmwasm_std::QuerierWrapper<'a, C>,
    _phantom: std::marker::PhantomData<Contract>,
}

impl<'a, C: cosmwasm_std::CustomQuery, Contract: ?Sized> BoundQuerier<'a, C, Contract> {
    /// Returns reference to the underlying [QuerierWrapper](cosmwasm_std::QuerierWrapper).
    pub fn querier(&self) -> &'a cosmwasm_std::QuerierWrapper<'a, C> {
        self.querier
    }

    /// Returns reference to the underlying contract address.
    pub fn contract(&self) -> &'a cosmwasm_std::Addr {
        self.contract
    }

    /// Creates a new instance of [BoundQuerier] from provided contract address and querier.
    pub fn borrowed(
        contract: &'a cosmwasm_std::Addr,
        querier: &'a cosmwasm_std::QuerierWrapper<'a, C>,
    ) -> Self {
        Self {
            contract,
            querier,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, C: cosmwasm_std::CustomQuery, Contract> From<&'a BoundQuerier<'a, C, Contract>>
    for BoundQuerier<'a, C, Contract>
{
    fn from(input: &'a BoundQuerier<'a, C, Contract>) -> Self {
        BoundQuerier::borrowed(input.contract, input.querier)
    }
}

pub struct EmptyExecutorBuilderState;
pub struct ReadyExecutorBuilderState;

/// This structure represents a collection of execution methods for
/// a Sylvia contract or interface. An instance of this structure
/// should be created using a [Remote] object, obtained through the
/// [Remote::executor] method.
///
/// [ExecutorBuilder] implements the
/// Executor traits generated by the `contract` and `interface` macros,
/// encompassing all `sv::msg(exec)` methods of the specified contracts
/// and interfaces.
///
/// ```rust
/// pub mod another_contract {
///     # use cosmwasm_std::{Response, StdResult};
///     # use sylvia::contract;
///     # use sylvia::types::{ExecCtx, InstantiateCtx};
///     pub struct AnotherContract {}
///     
///     #[contract]
///     impl AnotherContract {
///         pub fn new() -> Self { Self {} }
///
///         #[sv::msg(instantiate)]
///         fn instantiate(&self, ctx: InstantiateCtx) -> StdResult<Response> {
///             Ok(Response::new())
///         }
///
///         #[sv::msg(exec)]
///         fn exec_method(&self, ctx: ExecCtx) -> StdResult<Response> {
///             Ok(Response::new())
///         }
///     }
/// }
/// # use cosmwasm_std::{coin, Addr, Response, StdResult};
/// # use cw_storage_plus::Item;
/// # use sylvia::contract;
/// # use sylvia::types::{ExecCtx, InstantiateCtx, Remote};
/// # use another_contract::AnotherContract;
/// # use another_contract::sv::Executor;
///
/// pub struct Contract<'a> {
///    pub remote_contract: Item<Remote<'a, AnotherContract>>,
/// }
///
/// #[contract]
/// impl<'a> Contract<'a> {
///     pub fn new() -> Self {
///         Self {
///             remote_contract: Item::new("remote_contract"),
///         }
///     }
///
///     #[sv::msg(instantiate)]
///     fn instantiate(&self, ctx: InstantiateCtx, remote_addr: Addr) -> StdResult<Response> {
///         self.remote_contract.save(
///             ctx.deps.storage,
///             &Remote::new(remote_addr.clone()),
///         )?;
///         Ok(Response::new())
///     }
///
///     #[sv::msg(exec)]
///     fn call_another_contract(&self, ctx: ExecCtx) -> StdResult<Response> {
///         let remote = self.remote_contract.load(ctx.deps.storage)?;
///         let exec_method = remote.executor().with_funds(vec![coin(2345, "atom")]).exec_method()?.build();
///         Ok(Response::new().add_message(exec_method))
///     }
/// }
///
/// # fn main() {}
/// ```
///
/// It is also possible to call execution methods on other contract's interface:
///
/// ```rust
/// pub mod interface {
///     # use cosmwasm_std::{Response, StdResult, StdError};
///     # use sylvia::interface;
///     # use sylvia::types::ExecCtx;
///     #[interface]
///     pub trait Interface {
///         type Error: From<StdError>;
///
///         #[sv::msg(exec)]
///         fn exec_method(&self, ctx: ExecCtx) -> StdResult<Response>;
///     }
/// }
/// # use interface::Interface;
/// # use interface::sv::Executor;
/// # use cosmwasm_std::{Addr, Response, StdError, StdResult};
/// # use sylvia::types::Remote;
///
/// fn execute_method(remote_addr: Addr) -> StdResult<Response> {
///     let remote = Remote::<'_, dyn Interface<Error=StdError>>::new(remote_addr);
///     let msg = remote.executor().exec_method()?.build();
///     Ok(Response::new().add_message(msg))
/// }
/// ```
pub struct ExecutorBuilder<State: ?Sized> {
    contract: String,
    funds: Vec<Coin>,
    msg: Binary,
    _state: std::marker::PhantomData<State>,
}

impl<Contract: ?Sized> ExecutorBuilder<(EmptyExecutorBuilderState, Contract)> {
    /// Creates an instance of the structure based
    /// on the destination contract's address.
    pub fn new(contract: &cosmwasm_std::Addr) -> Self {
        Self {
            contract: contract.to_string(),
            funds: vec![],
            msg: Binary::default(),
            _state: std::marker::PhantomData,
        }
    }
}

impl<T, Contract: ?Sized> ExecutorBuilder<(T, Contract)> {
    /// Adds the funds to the execution message.
    pub fn with_funds(self, funds: Vec<Coin>) -> Self {
        Self { funds, ..self }
    }

    /// Returns funds set in this builder.
    pub fn funds(&self) -> &Vec<Coin> {
        &self.funds
    }

    /// Returns contract's address represented as [str].
    pub fn contract(&self) -> &str {
        &self.contract
    }
}

impl ExecutorBuilder<ReadyExecutorBuilderState> {
    pub fn new(contract: String, funds: Vec<Coin>, msg: Binary) -> Self {
        Self {
            contract,
            funds,
            msg,
            _state: std::marker::PhantomData,
        }
    }

    pub fn build(self) -> WasmMsg {
        WasmMsg::Execute {
            contract_addr: self.contract,
            msg: self.msg,
            funds: self.funds,
        }
    }
}

/// Represents a contract on the chain and acts as a gateway to communicate with it.
///
/// # Example
///
/// [Remote] stored as a contract field.
///
/// ```rust
/// mod admin_contract {
/// #   use sylvia::types::{InstantiateCtx, QueryCtx};
/// #   use sylvia::cw_std::{Addr, Response, StdResult};
/// #
///     pub struct AdminContract;
///
///     #[sylvia::contract]
///     impl AdminContract {
/// #       pub const fn new() -> Self {
/// #           Self
/// #       }
/// #
/// #       #[sv::msg(instantiate)]
/// #       pub fn instantiate(&self, ctx: InstantiateCtx) -> StdResult<Response> {
/// #           Ok(Response::new())
/// #       }
/// #
///         #[sv::msg(query)]
///         pub fn is_admin(&self, ctx: QueryCtx, addr: Addr) -> StdResult<bool> {
///             /// Validate admin
/// #           Ok(true)
///         }
///     }
/// }
///
/// mod other_contract {
///     use sylvia::types::{BoundQuerier, Remote};
///     use crate::admin_contract::AdminContract;
///     use crate::admin_contract::sv::Querier;
/// #    use sylvia::cw_std::{Addr, Deps, StdResult};
/// #    use cw_storage_plus::Item;
///
///     pub const ADMIN_CONTRACT: Item<Remote<AdminContract>> = Item::new("admin_contract");
///
///     fn is_admin(deps: Deps, addr: Addr) -> StdResult<bool> {
///         let admin_contract = ADMIN_CONTRACT.load(deps.storage)?;
///         admin_contract.querier(&deps.querier)
///             .is_admin(addr)
///     }
/// }
///
/// fn main() {}
/// ```
#[derive(Serialize, Deserialize, Derivative)]
#[derivative(Clone, Debug, PartialEq)]
pub struct Remote<'a, Contract: ?Sized> {
    addr: std::borrow::Cow<'a, cosmwasm_std::Addr>,
    #[serde(skip)]
    _phantom: std::marker::PhantomData<Contract>,
}

// Custom `JsonSchema` implementation to remove bounds for `Contract`.
impl<'a, Contract: ?Sized> schemars::JsonSchema for Remote<'a, Contract> {
    fn schema_name() -> std::string::String {
        "Remote".to_owned()
    }
    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        {
            let mut schema_object = schemars::schema::SchemaObject {
                instance_type: Some(schemars::schema::InstanceType::Object.into()),
                ..Default::default()
            };
            let object_validation = schema_object.object();
            {
                object_validation.properties.insert(
                    "addr".to_owned(),
                    gen.subschema_for::<std::borrow::Cow<'a, cosmwasm_std::Addr>>(),
                );
                if! <std::borrow::Cow<'a,cosmwasm_std::Addr>as schemars::JsonSchema> ::_schemars_private_is_option(){
            object_validation.required.insert("addr".to_owned());
          }
            }
            schemars::schema::Schema::Object(schema_object)
        }
    }
}

impl<'a, Contract: ?Sized> Remote<'a, Contract> {
    /// Creates a new instance of [Remote] from owned contract address.
    pub fn new(addr: cosmwasm_std::Addr) -> Self {
        Self {
            addr: std::borrow::Cow::Owned(addr),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Creates a new instance of [Remote] from borrowed contract address.
    pub fn borrowed(addr: &'a cosmwasm_std::Addr) -> Self {
        Self {
            addr: std::borrow::Cow::Borrowed(addr),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Creates a new instance of [BoundQuerier] from underlying contract address.
    pub fn querier<C: cosmwasm_std::CustomQuery>(
        &'a self,
        querier: &'a cosmwasm_std::QuerierWrapper<'a, C>,
    ) -> BoundQuerier<'a, C, Contract> {
        BoundQuerier {
            contract: &self.addr,
            querier,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn executor(&self) -> ExecutorBuilder<(EmptyExecutorBuilderState, Contract)> {
        ExecutorBuilder::<(EmptyExecutorBuilderState, Contract)>::new(&self.addr)
    }
}

impl<'a, Contract: ?Sized> AsRef<cosmwasm_std::Addr> for Remote<'a, Contract> {
    /// Returns reference to the underlying contract address.
    fn as_ref(&self) -> &cosmwasm_std::Addr {
        &self.addr
    }
}

/// Represantation of `reply` context received in entry point as
/// (DepsMut, Env) tuple.
pub struct ReplyCtx<'a, C: cosmwasm_std::CustomQuery = Empty> {
    pub deps: DepsMut<'a, C>,
    pub env: Env,
}

/// Represantation of `reply` context received in entry point as
/// (DepsMut, Env) tuple.
pub struct MigrateCtx<'a, C: cosmwasm_std::CustomQuery = Empty> {
    pub deps: DepsMut<'a, C>,
    pub env: Env,
}

/// Represantation of `reply` context received in entry point as
/// (DepsMut, Env, MessageInfo) tuple.
pub struct ExecCtx<'a, C: cosmwasm_std::CustomQuery = Empty> {
    pub deps: DepsMut<'a, C>,
    pub env: Env,
    pub info: MessageInfo,
}

/// Represantation of `instantiate` context received in entry point as
/// (DepsMut, Env, MessageInfo) tuple.
pub struct InstantiateCtx<'a, C: cosmwasm_std::CustomQuery = Empty> {
    pub deps: DepsMut<'a, C>,
    pub env: Env,
    pub info: MessageInfo,
}

/// Represantation of `query` context received in entry point as
/// (Deps, Env) tuple.
pub struct QueryCtx<'a, C: cosmwasm_std::CustomQuery = Empty> {
    pub deps: Deps<'a, C>,
    pub env: Env,
}

/// Represantation of `sudo` context received in entry point as
/// (DepsMut, Env) tuple.
pub struct SudoCtx<'a, C: cosmwasm_std::CustomQuery = Empty> {
    pub deps: DepsMut<'a, C>,
    pub env: Env,
}

impl<C: cosmwasm_std::CustomQuery> ExecCtx<'_, C> {
    pub fn branch(&'_ mut self) -> ExecCtx<'_, C> {
        ExecCtx {
            deps: self.deps.branch(),
            env: self.env.clone(),
            info: self.info.clone(),
        }
    }
}

impl<C: cosmwasm_std::CustomQuery> InstantiateCtx<'_, C> {
    pub fn branch(&'_ mut self) -> InstantiateCtx<'_, C> {
        InstantiateCtx {
            deps: self.deps.branch(),
            env: self.env.clone(),
            info: self.info.clone(),
        }
    }
}

impl<C: cosmwasm_std::CustomQuery> SudoCtx<'_, C> {
    pub fn branch(&'_ mut self) -> SudoCtx<'_, C> {
        SudoCtx {
            deps: self.deps.branch(),
            env: self.env.clone(),
        }
    }
}

impl<'a, C: cosmwasm_std::CustomQuery> From<(DepsMut<'a, C>, Env)> for MigrateCtx<'a, C> {
    fn from((deps, env): (DepsMut<'a, C>, Env)) -> Self {
        Self { deps, env }
    }
}

impl<'a, C: cosmwasm_std::CustomQuery> From<(DepsMut<'a, C>, Env)> for ReplyCtx<'a, C> {
    fn from((deps, env): (DepsMut<'a, C>, Env)) -> Self {
        Self { deps, env }
    }
}

impl<'a, C: cosmwasm_std::CustomQuery> From<(DepsMut<'a, C>, Env, MessageInfo)> for ExecCtx<'a, C> {
    fn from((deps, env, info): (DepsMut<'a, C>, Env, MessageInfo)) -> Self {
        Self { deps, env, info }
    }
}

impl<'a, C: cosmwasm_std::CustomQuery> From<(DepsMut<'a, C>, Env, MessageInfo)>
    for InstantiateCtx<'a, C>
{
    fn from((deps, env, info): (DepsMut<'a, C>, Env, MessageInfo)) -> Self {
        Self { deps, env, info }
    }
}

impl<'a, C: cosmwasm_std::CustomQuery> From<(Deps<'a, C>, Env)> for QueryCtx<'a, C> {
    fn from((deps, env): (Deps<'a, C>, Env)) -> Self {
        Self { deps, env }
    }
}

impl<'a, C: cosmwasm_std::CustomQuery> From<(DepsMut<'a, C>, Env)> for SudoCtx<'a, C> {
    fn from((deps, env): (DepsMut<'a, C>, Env)) -> Self {
        Self { deps, env }
    }
}

/// Set of trait bounds for custom messages.
pub trait CustomMsg: cosmwasm_std::CustomMsg + DeserializeOwned {}

impl<T> CustomMsg for T where T: cosmwasm_std::CustomMsg + DeserializeOwned {}

/// Set of trait bounds for custom queries.
pub trait CustomQuery: cosmwasm_std::CustomQuery + DeserializeOwned + JsonSchema {}

impl<T> CustomQuery for T where T: cosmwasm_std::CustomQuery + DeserializeOwned + JsonSchema {}

/// Api trait for easier access to generated types and messages.
pub trait InterfaceApi {
    type Exec;
    type Query;
    type Sudo;
    type Querier<'querier, Contract>;
}

/// Api trait for easier access to generated types and messages.
pub trait ContractApi {
    type Instantiate;
    type Query;
    type Exec;
    type ContractQuery;
    type ContractExec;
    type ContractSudo;
    type Migrate;
    type Sudo;
    type Querier<'querier>;
    type Remote<'remote>;
}

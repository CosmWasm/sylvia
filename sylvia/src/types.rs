//! Module providing utilities to build and use sylvia contracts.
use cosmwasm_std::{Deps, DepsMut, Empty, Env, MessageInfo};
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
pub struct BoundQuerier<'a, C: cosmwasm_std::CustomQuery, Contract> {
    contract: &'a cosmwasm_std::Addr,
    querier: &'a cosmwasm_std::QuerierWrapper<'a, C>,
    _phantom: std::marker::PhantomData<Contract>,
}

impl<'a, C: cosmwasm_std::CustomQuery, Contract> BoundQuerier<'a, C, Contract> {
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Remote<'a, Contract> {
    addr: std::borrow::Cow<'a, cosmwasm_std::Addr>,
    #[serde(skip)]
    _phantom: std::marker::PhantomData<Contract>,
}

impl<'a, Contract> Remote<'a, Contract> {
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
}

impl<'a, Contract> AsRef<cosmwasm_std::Addr> for Remote<'a, Contract> {
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

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Deps, DepsMut, Empty, Env, MessageInfo};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;

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

#[cfg(not(tarpaulin_include))]
impl<C: cosmwasm_std::CustomQuery> ExecCtx<'_, C> {
    pub fn branch(&'_ mut self) -> ExecCtx<'_, C> {
        ExecCtx {
            deps: self.deps.branch(),
            env: self.env.clone(),
            info: self.info.clone(),
        }
    }
}

#[cfg(not(tarpaulin_include))]
impl<C: cosmwasm_std::CustomQuery> InstantiateCtx<'_, C> {
    pub fn branch(&'_ mut self) -> InstantiateCtx<'_, C> {
        InstantiateCtx {
            deps: self.deps.branch(),
            env: self.env.clone(),
            info: self.info.clone(),
        }
    }
}

#[cfg(not(tarpaulin_include))]
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

impl<'a, C: CustomQuery> From<(DepsMut<'a, C>, Env)> for SudoCtx<'a, C> {
    fn from((deps, env): (DepsMut<'a, C>, Env)) -> Self {
        Self { deps, env }
    }
}

pub trait CustomMsg: cosmwasm_std::CustomMsg + DeserializeOwned {}

impl<T> CustomMsg for T where T: cosmwasm_std::CustomMsg + DeserializeOwned {}

pub trait CustomQuery: cosmwasm_std::CustomQuery + DeserializeOwned + JsonSchema {}

impl<T> CustomQuery for T where T: cosmwasm_std::CustomQuery + DeserializeOwned + JsonSchema {}

/// This type is meant to be used in internal testing purpose.
#[cw_serde]
pub struct SvCustomMsg;

impl cosmwasm_std::CustomMsg for SvCustomMsg {}

/// This type is meant to be used in internal testing purpose.
#[cw_serde]
pub struct SvCustomQuery;

impl cosmwasm_std::CustomQuery for SvCustomQuery {}

/// Api trait for easier access to generated types and messages.
pub trait InterfaceApi {
    type Exec;
    type Query;
    type Querier<'querier>;
}

/// Api trait for easier access to generated types and messages.
pub trait ContractApi {
    type Instantiate;
    type Query;
    type Exec;
    type ContractQuery;
    type ContractExec;
    type Migrate;
    type Querier<'querier>;
    type Remote<'remote>;
}

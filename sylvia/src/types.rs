use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CustomQuery, Deps, DepsMut, Empty, Env, MessageInfo};
use serde::de::DeserializeOwned;

pub struct ReplyCtx<'a, C: CustomQuery = Empty> {
    pub deps: DepsMut<'a, C>,
    pub env: Env,
}

pub struct MigrateCtx<'a, C: CustomQuery = Empty> {
    pub deps: DepsMut<'a, C>,
    pub env: Env,
}

pub struct ExecCtx<'a, C: CustomQuery = Empty> {
    pub deps: DepsMut<'a, C>,
    pub env: Env,
    pub info: MessageInfo,
}

pub struct InstantiateCtx<'a, C: CustomQuery = Empty> {
    pub deps: DepsMut<'a, C>,
    pub env: Env,
    pub info: MessageInfo,
}

pub struct QueryCtx<'a, C: CustomQuery = Empty> {
    pub deps: Deps<'a, C>,
    pub env: Env,
}

pub struct SudoCtx<'a, C: CustomQuery = Empty> {
    pub deps: DepsMut<'a, C>,
    pub env: Env,
}

#[cfg(not(tarpaulin_include))]
impl<C: CustomQuery> ExecCtx<'_, C> {
    pub fn branch(&'_ mut self) -> ExecCtx<'_, C> {
        ExecCtx {
            deps: self.deps.branch(),
            env: self.env.clone(),
            info: self.info.clone(),
        }
    }
}

#[cfg(not(tarpaulin_include))]
impl<C: CustomQuery> InstantiateCtx<'_, C> {
    pub fn branch(&'_ mut self) -> InstantiateCtx<'_, C> {
        InstantiateCtx {
            deps: self.deps.branch(),
            env: self.env.clone(),
            info: self.info.clone(),
        }
    }
}

#[cfg(not(tarpaulin_include))]
impl<C: CustomQuery> SudoCtx<'_, C> {
    pub fn branch(&'_ mut self) -> SudoCtx<'_, C> {
        SudoCtx {
            deps: self.deps.branch(),
            env: self.env.clone(),
        }
    }
}

impl<'a, C: CustomQuery> From<(DepsMut<'a, C>, Env)> for MigrateCtx<'a, C> {
    fn from((deps, env): (DepsMut<'a, C>, Env)) -> Self {
        Self { deps, env }
    }
}

impl<'a, C: CustomQuery> From<(DepsMut<'a, C>, Env)> for ReplyCtx<'a, C> {
    fn from((deps, env): (DepsMut<'a, C>, Env)) -> Self {
        Self { deps, env }
    }
}

impl<'a, C: CustomQuery> From<(DepsMut<'a, C>, Env, MessageInfo)> for ExecCtx<'a, C> {
    fn from((deps, env, info): (DepsMut<'a, C>, Env, MessageInfo)) -> Self {
        Self { deps, env, info }
    }
}

impl<'a, C: CustomQuery> From<(DepsMut<'a, C>, Env, MessageInfo)> for InstantiateCtx<'a, C> {
    fn from((deps, env, info): (DepsMut<'a, C>, Env, MessageInfo)) -> Self {
        Self { deps, env, info }
    }
}

impl<'a, C: CustomQuery> From<(Deps<'a, C>, Env)> for QueryCtx<'a, C> {
    fn from((deps, env): (Deps<'a, C>, Env)) -> Self {
        Self { deps, env }
    }
}

pub trait CustomMsg: cosmwasm_std::CustomMsg + DeserializeOwned {}

impl<T> CustomMsg for T where T: cosmwasm_std::CustomMsg + DeserializeOwned {}

#[cw_serde]
pub struct SvCustomMsg;

impl cosmwasm_std::CustomMsg for SvCustomMsg {}

pub trait InterfaceMessages {
    type Exec;
    type Query;
}

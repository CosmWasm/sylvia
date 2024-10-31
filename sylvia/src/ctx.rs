use cosmwasm_std::{Deps, DepsMut, Empty, Env, Event, MessageInfo, MsgResponse};

/// Represantation of `reply` context received in entry point.
#[non_exhaustive]
pub struct ReplyCtx<'a, C: cosmwasm_std::CustomQuery = Empty> {
    pub deps: DepsMut<'a, C>,
    pub env: Env,
    pub gas_used: u64,
    pub events: Vec<Event>,
    pub msg_responses: Vec<MsgResponse>,
}

/// Represantation of `migrate` context received in entry point.
#[non_exhaustive]
pub struct MigrateCtx<'a, C: cosmwasm_std::CustomQuery = Empty> {
    pub deps: DepsMut<'a, C>,
    pub env: Env,
}

/// Represantation of `execute` context received in entry point.
#[non_exhaustive]
pub struct ExecCtx<'a, C: cosmwasm_std::CustomQuery = Empty> {
    pub deps: DepsMut<'a, C>,
    pub env: Env,
    pub info: MessageInfo,
}

/// Represantation of `instantiate` context received in entry point.
#[non_exhaustive]
pub struct InstantiateCtx<'a, C: cosmwasm_std::CustomQuery = Empty> {
    pub deps: DepsMut<'a, C>,
    pub env: Env,
    pub info: MessageInfo,
}

/// Represantation of `query` context received in entry point.
#[non_exhaustive]
pub struct QueryCtx<'a, C: cosmwasm_std::CustomQuery = Empty> {
    pub deps: Deps<'a, C>,
    pub env: Env,
}

/// Represantation of `sudo` context received in entry point.
#[non_exhaustive]
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

impl<'a, C: cosmwasm_std::CustomQuery>
    From<(DepsMut<'a, C>, Env, u64, Vec<Event>, Vec<MsgResponse>)> for ReplyCtx<'a, C>
{
    fn from(
        (deps, env, gas_used, events, msg_responses): (
            DepsMut<'a, C>,
            Env,
            u64,
            Vec<Event>,
            Vec<MsgResponse>,
        ),
    ) -> Self {
        Self {
            deps,
            env,
            gas_used,
            events,
            msg_responses,
        }
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

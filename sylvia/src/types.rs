use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo};

pub struct ReplyCtx<'a> {
    pub deps: DepsMut<'a>,
    pub env: Env,
}

pub struct MigrateCtx<'a> {
    pub deps: DepsMut<'a>,
    pub env: Env,
}

pub struct ExecCtx<'a> {
    pub deps: DepsMut<'a>,
    pub env: Env,
    pub info: MessageInfo,
}

pub type InstantiateCtx<'a> = ExecCtx<'a>;

pub struct QueryCtx<'a> {
    pub deps: Deps<'a>,
    pub env: Env,
}

impl ExecCtx<'_> {
    pub fn branch(&'_ mut self) -> ExecCtx<'_> {
        ExecCtx {
            deps: self.deps.branch(),
            env: self.env.clone(),
            info: self.info.clone(),
        }
    }
}

impl<'a> From<(DepsMut<'a>, Env)> for ReplyCtx<'a> {
    fn from((deps, env): (DepsMut<'a>, Env)) -> Self {
        Self { deps, env }
    }
}

impl<'a> From<(DepsMut<'a>, Env)> for MigrateCtx<'a> {
    fn from((deps, env): (DepsMut<'a>, Env)) -> Self {
        Self { deps, env }
    }
}

impl<'a> From<(DepsMut<'a>, Env, MessageInfo)> for ExecCtx<'a> {
    fn from((deps, env, info): (DepsMut<'a>, Env, MessageInfo)) -> Self {
        Self { deps, env, info }
    }
}

impl<'a> From<(Deps<'a>, Env)> for QueryCtx<'a> {
    fn from((deps, env): (Deps<'a>, Env)) -> Self {
        Self { deps, env }
    }
}

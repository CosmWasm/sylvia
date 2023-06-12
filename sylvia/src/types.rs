use cosmwasm_std::{CustomQuery, Deps, DepsMut, Empty, Env, MessageInfo};

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

impl<C: CustomQuery> ExecCtx<'_, C> {
    pub fn branch(&'_ mut self) -> ExecCtx<'_, C> {
        ExecCtx {
            deps: self.deps.branch(),
            env: self.env.clone(),
            info: self.info.clone(),
        }
    }
}
impl<C: CustomQuery> InstantiateCtx<'_, C> {
    pub fn branch(&'_ mut self) -> InstantiateCtx<'_, C> {
        InstantiateCtx {
            deps: self.deps.branch(),
            env: self.env.clone(),
            info: self.info.clone(),
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

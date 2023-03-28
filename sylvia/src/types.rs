use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo};

pub struct MigrationCtx<'a> {
    deps: DepsMut<'a>,
    env: Env,
}

pub struct InstantiateCtx<'a> {
    deps: DepsMut<'a>,
    env: Env,
    info: MessageInfo,
}

pub struct ExecCtx<'a> {
    deps: DepsMut<'a>,
    env: Env,
    info: MessageInfo,
}

pub struct QueryCtx<'a> {
    deps: Deps<'a>,
    env: Env,
}

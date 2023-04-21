use anyhow::Error;
use cosmwasm_std::{Addr, Response, StdError};
use cw_storage_plus::{Item, Map};
use sylvia::types::{ExecCtx, InstantiateCtx, QueryCtx};
use sylvia::{contract, interface};

#[derive(
    sylvia::serde::Serialize,
    sylvia::serde::Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    sylvia::schemars::JsonSchema,
)]
pub struct Member {
    addr: String,
    weight: u64,
}

#[derive(
    sylvia::serde::Serialize,
    sylvia::serde::Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    sylvia::schemars::JsonSchema,
)]
pub struct MemberResp {
    weight: u64,
}

#[interface(module=group)]
pub trait Group {
    type Error: From<StdError>;

    #[msg(exec)]
    fn update_admin(&self, ctx: ExecCtx, admin: Option<String>) -> Result<Response, Self::Error>;

    #[msg(exec)]
    fn update_members(
        &self,
        ctx: ExecCtx,
        remove: Vec<String>,
        add: Vec<Member>,
    ) -> Result<Response, Self::Error>;

    #[msg(query)]
    fn member(&self, ctx: QueryCtx, addr: String) -> Result<MemberResp, Self::Error>;
}

pub struct GroupContract {
    admin: Item<'static, Addr>,
    _members: Map<'static, Addr, u64>,
}

impl Default for GroupContract {
    fn default() -> Self {
        Self::new()
    }
}

#[contract]
impl Group for GroupContract {
    type Error = Error;

    #[msg(exec)]
    fn update_admin(&self, _ctx: ExecCtx, _admin: Option<String>) -> Result<Response, Self::Error> {
        todo!()
    }

    #[msg(exec)]
    fn update_members(
        &self,
        _ctx: ExecCtx,
        _remove: Vec<String>,
        _add: Vec<Member>,
    ) -> Result<Response, Self::Error> {
        todo!()
    }

    #[msg(query)]
    fn member(&self, _ctx: QueryCtx, _addr: String) -> Result<MemberResp, Self::Error> {
        todo!()
    }
}

#[contract(module=contract)]
#[error(Error)]
#[messages(group as Group)]
impl GroupContract {
    pub const fn new() -> Self {
        Self {
            admin: Item::new("admin"),
            _members: Map::new("members"),
        }
    }

    #[msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: InstantiateCtx,
        admin: Option<String>,
    ) -> Result<Response, Error> {
        if let Some(admin) = admin {
            let admin = ctx.deps.api.addr_validate(&admin)?;
            self.admin.save(ctx.deps.storage, &admin)?;
        }

        Ok(Response::new())
    }
}

fn main() {}

use anyhow::Error;
use cosmwasm_std::{Addr, Response};
use cw_storage_plus::{Item, Map};
use sylvia::contract;
use sylvia::types::InstantiateCtx;

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

mod group {
    use cosmwasm_std::{Response, StdError};
    use sylvia::interface;
    use sylvia::types::{ExecCtx, QueryCtx};

    use crate::{Member, MemberResp};

    #[interface]
    pub trait Group {
        type Error: From<StdError>;

        #[msg(exec)]
        fn update_admin(
            &self,
            ctx: ExecCtx,
            admin: Option<String>,
        ) -> Result<Response, Self::Error>;

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
}

mod impl_group {
    use anyhow::Error;
    use cosmwasm_std::Response;
    use sylvia::types::{ExecCtx, QueryCtx};
    use sylvia_derive::contract;

    use crate::{GroupContract, MemberResp};

    #[contract(module=crate)]
    #[messages(crate::group as Group)]
    impl crate::group::Group for GroupContract {
        type Error = Error;

        #[msg(exec)]
        fn update_admin(
            &self,
            _ctx: ExecCtx,
            _admin: Option<String>,
        ) -> Result<Response, Self::Error> {
            todo!()
        }

        #[msg(exec)]
        fn update_members(
            &self,
            _ctx: ExecCtx,
            _remove: Vec<String>,
            _add: Vec<crate::Member>,
        ) -> Result<Response, Self::Error> {
            todo!()
        }

        #[msg(query)]
        fn member(&self, _ctx: QueryCtx, _addr: String) -> Result<MemberResp, Self::Error> {
            todo!()
        }
    }
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

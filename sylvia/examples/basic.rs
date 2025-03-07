use anyhow::Error;
use cosmwasm_std::{Addr, Response};
use cw_storage_plus::{Item, Map};
use sylvia::contract;
use sylvia::ctx::InstantiateCtx;

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
    use sylvia::ctx::{ExecCtx, QueryCtx};
    use sylvia::interface;

    use crate::{Member, MemberResp};

    #[interface]
    #[sv::custom(msg=cosmwasm_std::Empty, query=cosmwasm_std::Empty)]
    pub trait Group {
        type Error: From<StdError>;

        #[sv::msg(exec)]
        fn update_admin(
            &self,
            ctx: ExecCtx,
            admin: Option<String>,
        ) -> Result<Response, Self::Error>;

        #[sv::msg(exec)]
        fn update_members(
            &self,
            ctx: ExecCtx,
            remove: Vec<String>,
            add: Vec<Member>,
        ) -> Result<Response, Self::Error>;

        #[sv::msg(query)]
        fn member(&self, ctx: QueryCtx, addr: String) -> Result<MemberResp, Self::Error>;
    }
}

mod impl_group {
    use anyhow::Error;
    use cosmwasm_std::Response;
    use sylvia::ctx::{ExecCtx, QueryCtx};

    use crate::{GroupContract, MemberResp};

    impl crate::group::Group for GroupContract {
        type Error = Error;

        fn update_admin(
            &self,
            _ctx: ExecCtx,
            _admin: Option<String>,
        ) -> Result<Response, Self::Error> {
            todo!()
        }

        fn update_members(
            &self,
            _ctx: ExecCtx,
            _remove: Vec<String>,
            _add: Vec<crate::Member>,
        ) -> Result<Response, Self::Error> {
            todo!()
        }

        fn member(&self, _ctx: QueryCtx, _addr: String) -> Result<MemberResp, Self::Error> {
            todo!()
        }
    }
}

pub struct GroupContract {
    admin: Item<Addr>,
    _members: Map<Addr, u64>,
}

impl Default for GroupContract {
    fn default() -> Self {
        Self::new()
    }
}

#[contract]
#[sv::error(Error)]
#[sv::messages(group as Group)]
impl GroupContract {
    pub const fn new() -> Self {
        Self {
            admin: Item::new("admin"),
            _members: Map::new("members"),
        }
    }

    #[sv::msg(instantiate)]
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

use anyhow::Error;
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdError};
use cw1::*;
use cw1::{msg::ExecMsg, msg::QueryMsg, Cw1};
use cw_storage_plus::{Item, Map};
use sylvia::contract;

pub struct Cw1Whitelist {
    members: Map<'static, Addr, u64>,
}

impl Default for Cw1Whitelist {
    fn default() -> Self {
        Self::new()
    }
}

impl Cw1 for Cw1Whitelist {
    type Error = Error;
    fn add_member(
        &self,
        _ctx: (
            cosmwasm_std::DepsMut,
            cosmwasm_std::Env,
            cosmwasm_std::MessageInfo,
        ),
        _member: String,
    ) -> Result<Response, Self::Error> {
        todo!()
    }
    fn find_member(
        &self,
        _ctx: (cosmwasm_std::Deps, cosmwasm_std::Env),
        _member: String,
    ) -> Result<Response, Self::Error> {
        todo!()
    }
}

#[contract(module=contract, error=Error)]
#[messages(msg as Cw1)]
impl Cw1Whitelist {
    pub fn new() -> Self {
        Self {
            members: Map::new("members"),
        }
    }

    #[msg(instantiate)]
    pub fn instantiate(
        &self,
        (deps, _env, _msg): (DepsMut, Env, MessageInfo),
        members: Option<String>,
    ) -> Result<Response, Error> {
        // if let Some(admin) = admin {
        //     let admin = deps.api.addr_validate(&admin)?;
        //     self.admin.save(deps.storage, &admin)?;
        // }

        Ok(Response::new())
    }
}

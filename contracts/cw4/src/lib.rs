use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdError};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sylvia::interface;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct FindMemberResponse {
    pub is_present: bool,
}

#[interface]
pub trait Cw4 {
    type Error: From<StdError>;

    #[msg(exec)]
    fn update_admin(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        admin: String,
    ) -> Result<Response, Self::Error>;

    #[msg(exec)]
    fn update_members(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        members: Vec<String>,
    ) -> Result<Response, Self::Error>;

    #[msg(exec)]
    fn add_hook(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        hook: String,
    ) -> Result<Response, Self::Error>;

    #[msg(exec)]
    fn remove_hook(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        hook: String,
    ) -> Result<Response, Self::Error>;

    #[msg(query)]
    fn member(&self, ctx: (Deps, Env), member: String) -> Result<Response, Self::Error>;

    #[msg(query)]
    fn list_members(&self, ctx: (Deps, Env)) -> Result<Response, Self::Error>;

    #[msg(query)]
    fn total_weight(&self, ctx: (Deps, Env)) -> Result<Response, Self::Error>;

    #[msg(query)]
    fn admin(&self, ctx: (Deps, Env)) -> Result<Response, Self::Error>;

    #[msg(query)]
    fn hooks(&self, ctx: (Deps, Env)) -> Result<Response, Self::Error>;
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{from_binary, from_slice, to_binary};

    use super::*;

    #[test]
    fn execute() {
        let original_msg = ExecMsg::UpdateAdmin {
            admin: "admin_name".to_owned(),
        };

        let serialized_msg = to_binary(&original_msg).unwrap();
        let serialized_msg: ExecMsg = from_binary(&serialized_msg).unwrap();

        assert_eq!(serialized_msg, original_msg);
    }

    #[test]
    fn query() {
        let original_msg = QueryMsg::Admin {};

        let serialized_msg = to_binary(&original_msg).unwrap();
        let serialized_msg: QueryMsg = from_binary(&serialized_msg).unwrap();

        assert_eq!(serialized_msg, original_msg);
    }

    #[test]
    fn execute_from_slice() {
        let deserialized: ExecMsg =
            from_slice(br#"{"update_admin": {"admin": "admin_name"}}"#).unwrap();
        assert_eq!(
            deserialized,
            ExecMsg::UpdateAdmin {
                admin: "admin_name".to_owned()
            }
        );
    }

    #[test]
    fn query_from_slice() {
        let deserialized: QueryMsg = from_slice(br#"{"admin": {}}"#).unwrap();
        assert_eq!(deserialized, QueryMsg::Admin {});
    }

    #[test]
    fn exec_msgs() {
        assert_eq!(
            ExecMsg::messages(),
            ["add_hook", "remove_hook", "update_admin", "update_members"]
        );
    }

    #[test]
    fn query_msgs() {
        assert_eq!(
            QueryMsg::messages(),
            ["admin", "hooks", "list_members", "member", "total_weight"]
        );
    }
}

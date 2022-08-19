#![allow(dead_code)]
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdError};

use sylvia::interface;

#[interface(module=msg)]
pub trait Cw1 {
    type Error: From<StdError>;

    #[msg(exec)]
    fn add_member(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        member: String,
    ) -> Result<Response, Self::Error>;

    #[msg(query)]
    fn find_member(&self, ctx: (Deps, Env), member: String) -> Result<Response, Self::Error>;
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{from_binary, from_slice, to_binary};

    use super::*;

    #[test]
    fn execute() {
        let original_msg = msg::ExecMsg::AddMember {
            member: "member_name".to_owned(),
        };

        let serialized_msg = to_binary(&original_msg).unwrap();
        let serialized_msg: msg::ExecMsg = from_binary(&serialized_msg).unwrap();

        assert_eq!(serialized_msg, original_msg);
    }

    #[test]
    fn query() {
        let original_msg = msg::QueryMsg::FindMember {
            member: "member_name".to_owned(),
        };

        let serialized_msg = to_binary(&original_msg).unwrap();
        let serialized_msg: msg::QueryMsg = from_binary(&serialized_msg).unwrap();

        assert_eq!(serialized_msg, original_msg);
    }

    #[test]
    fn execute_from_slice() {
        let deserialized: msg::ExecMsg =
            from_slice(br#"{"add_member": {"member": "some_member"}}"#).unwrap();
        assert_eq!(
            deserialized,
            msg::ExecMsg::AddMember {
                member: "some_member".to_owned()
            }
        );
    }

    #[test]
    fn query_from_slice() {
        let deserialized: msg::QueryMsg =
            from_slice(br#"{"find_member": {"member": "some_member"}}"#).unwrap();
        assert_eq!(
            deserialized,
            msg::QueryMsg::FindMember {
                member: "some_member".to_owned()
            }
        );
    }
}
#![allow(dead_code)]
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdError};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sylvia::interface;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct FindMemberResponse {
    pub is_present: bool,
}

#[interface]
pub trait Cw1 {
    type Error: From<StdError>;

    #[msg(exec)]
    fn add_member(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        member: String,
    ) -> Result<Response, Self::Error>;

    #[msg(query)]
    fn find_member(
        &self,
        ctx: (Deps, Env),
        member: String,
    ) -> Result<FindMemberResponse, Self::Error>;
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{from_binary, from_slice, to_binary};

    use super::*;

    #[test]
    fn execute() {
        let original_msg = ExecMsg::AddMember {
            member: "member_name".to_owned(),
        };

        let serialized_msg = to_binary(&original_msg).unwrap();
        let serialized_msg: ExecMsg = from_binary(&serialized_msg).unwrap();

        assert_eq!(serialized_msg, original_msg);
    }

    #[test]
    fn query() {
        let original_msg = QueryMsg::FindMember {
            member: "member_name".to_owned(),
        };

        let serialized_msg = to_binary(&original_msg).unwrap();
        let serialized_msg: QueryMsg = from_binary(&serialized_msg).unwrap();

        assert_eq!(serialized_msg, original_msg);
    }

    #[test]
    fn execute_from_slice() {
        let deserialized: ExecMsg =
            from_slice(br#"{"add_member": {"member": "some_member"}}"#).unwrap();
        assert_eq!(
            deserialized,
            ExecMsg::AddMember {
                member: "some_member".to_owned()
            }
        );
    }

    #[test]
    fn query_from_slice() {
        let deserialized: QueryMsg =
            from_slice(br#"{"find_member": {"member": "some_member"}}"#).unwrap();
        assert_eq!(
            deserialized,
            QueryMsg::FindMember {
                member: "some_member".to_owned()
            }
        );
    }
}

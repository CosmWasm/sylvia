use cosmwasm_std::{CosmosMsg, Response, StdResult};
use responses::AdminListResponse;
use sylvia::types::{ExecCtx, QueryCtx};
use sylvia::{interface, schemars};

pub mod responses;

#[interface]
#[sv::custom(msg=cosmwasm_std::Empty, query=cosmwasm_std::Empty)]
pub trait Whitelist {
    type Error: From<cosmwasm_std::StdError>;

    #[sv::msg(exec)]
    fn freeze(&self, ctx: ExecCtx) -> Result<Response, Self::Error>;

    #[sv::msg(exec)]
    fn update_admins(&self, ctx: ExecCtx, admins: Vec<String>) -> Result<Response, Self::Error>;

    #[sv::msg(query)]
    fn admin_list(&self, ctx: QueryCtx) -> StdResult<AdminListResponse>;

    /// Execute requests the contract to re-dispatch all these messages with the
    /// contract's address as sender. Every implementation has it's own logic to
    /// determine in
    #[sv::msg(exec)]
    fn execute2(&self, ctx: ExecCtx, msgs: Vec<CosmosMsg>) -> Result<Response, Self::Error>;
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{from_json, to_json_binary};

    use super::*;

    #[test]
    fn execs() {
        let original = sv::ExecMsg::Freeze {};

        let serialized = to_json_binary(&original).unwrap();
        let deserialized = from_json(serialized).unwrap();

        assert_eq!(original, deserialized);

        let original = sv::ExecMsg::UpdateAdmins {
            admins: vec!["new_admin".to_owned()],
        };

        let serialized = to_json_binary(&original).unwrap();
        let deserialized = from_json(serialized).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn execs_from_json() {
        let deserialized = from_json(br#"{"freeze": { }}"#).unwrap();
        assert_eq!(sv::ExecMsg::Freeze {}, deserialized);

        let deserialized = from_json(br#"{"update_admins": { "admins": ["new_admin"] }}"#).unwrap();
        assert_eq!(
            sv::ExecMsg::UpdateAdmins {
                admins: vec!["new_admin".to_owned()]
            },
            deserialized
        );
    }

    #[test]
    fn query() {
        let original = sv::QueryMsg::AdminList {};
        let serialized = to_json_binary(&original).unwrap();
        let deserialized = from_json(serialized).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn query_from_json() {
        let deserialized = from_json(br#"{"admin_list": {}}"#).unwrap();
        assert_eq!(sv::QueryMsg::AdminList {}, deserialized);
    }
}

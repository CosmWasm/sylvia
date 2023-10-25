use cosmwasm_std::{Response, StdResult};
use responses::AdminListResponse;
use sylvia::types::{ExecCtx, QueryCtx};
use sylvia::{interface, schemars};

pub mod responses;

#[interface]
pub trait Whitelist {
    type Error: From<cosmwasm_std::StdError>;

    #[msg(exec)]
    fn freeze(&self, ctx: ExecCtx) -> Result<Response, Self::Error>;

    #[msg(exec)]
    fn update_admins(&self, ctx: ExecCtx, admins: Vec<String>) -> Result<Response, Self::Error>;

    #[msg(query)]
    fn admin_list(&self, ctx: QueryCtx) -> StdResult<AdminListResponse>;
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{from_binary, from_slice, to_binary};

    use super::*;

    #[test]
    fn exec_from_binary() {
        let original = sv::ExecMsg::Freeze {};

        let serialized = to_binary(&original).unwrap();
        let deserialized = from_binary(&serialized).unwrap();

        assert_eq!(original, deserialized);

        let original = sv::ExecMsg::UpdateAdmins {
            admins: vec!["new_admin".to_owned()],
        };

        let serialized = to_binary(&original).unwrap();
        let deserialized = from_binary(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn exec_from_slice() {
        let deserialized = from_slice(br#"{"freeze": { }}"#).unwrap();
        assert_eq!(sv::ExecMsg::Freeze {}, deserialized);

        let deserialized =
            from_slice(br#"{"update_admins": { "admins": ["new_admin"] }}"#).unwrap();
        assert_eq!(
            sv::ExecMsg::UpdateAdmins {
                admins: vec!["new_admin".to_owned()]
            },
            deserialized
        );
    }

    #[test]
    fn query_from_binary() {
        let original = sv::QueryMsg::AdminList {};
        let serialized = to_binary(&original).unwrap();
        let deserialized = from_binary(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn query_from_slice() {
        let deserialized = from_slice(br#"{"admin_list": {}}"#).unwrap();
        assert_eq!(sv::QueryMsg::AdminList {}, deserialized);
    }
}

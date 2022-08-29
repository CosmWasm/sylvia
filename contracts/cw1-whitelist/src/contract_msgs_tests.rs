#[cfg(test)]
mod tests {
    use crate::error::ContractError;
    use cosmwasm_std::{
        from_binary, from_slice, to_binary, Addr, Deps, DepsMut, Empty, Env, MessageInfo, Response,
    };

    use cw1::Cw1;
    use cw1::*;
    use cw_storage_plus::Map;
    use sylvia::contract;

    pub struct Cw1TestContract {
        members: Map<'static, Addr, Empty>,
    }

    impl Cw1 for Cw1TestContract {
        type Error = ContractError;
        fn add_member(
            &self,
            (deps, _env, _info): (
                cosmwasm_std::DepsMut,
                cosmwasm_std::Env,
                cosmwasm_std::MessageInfo,
            ),
            member: String,
        ) -> Result<Response, Self::Error> {
            self.members
                .save(deps.storage, deps.api.addr_validate(&member)?, &Empty {})?;

            Ok(Response::new())
        }

        fn find_member(
            &self,
            (deps, _env): (cosmwasm_std::Deps, cosmwasm_std::Env),
            member: String,
        ) -> Result<FindMemberResponse, Self::Error> {
            let is_present = self
                .members
                .has(deps.storage, deps.api.addr_validate(&member)?);

            Ok(FindMemberResponse { is_present })
        }
    }

    #[contract(error=ContractError)]
    #[messages(cw1 as Cw1)]
    impl Cw1TestContract {
        #[allow(dead_code)]
        pub const fn new() -> Self {
            Self {
                members: Map::new("members"),
            }
        }

        #[allow(dead_code)]
        #[msg(instantiate)]
        pub fn instantiate(
            &self,
            (deps, _env, _msg): (DepsMut, Env, MessageInfo),
            members: Vec<String>,
        ) -> Result<Response, ContractError> {
            for addr in members.into_iter() {
                self.members
                    .save(deps.storage, deps.api.addr_validate(&addr)?, &Empty {})?;
            }

            Ok(Response::new())
        }
        #[allow(dead_code)]
        #[msg(exec)]
        fn remove_member(
            &self,
            (deps, _env, _msg): (DepsMut, Env, MessageInfo),
            member: String,
        ) -> Result<Response, ContractError> {
            self.members
                .remove(deps.storage, deps.api.addr_validate(&member)?);

            Ok(Response::new())
        }
        #[allow(dead_code)]
        #[allow(unused_variables)]
        #[msg(query)]
        fn query(&self, _ctx: (Deps, Env), member: String) -> Result<Response, ContractError> {
            Ok(Response::new())
        }
    }

    #[test]
    fn binary_serialize_exec() {
        let original_msg = ImplExecMsg::RemoveMember {
            member: "member".to_owned(),
        };

        let serialized_msg = to_binary(&original_msg).unwrap();
        let serialized_msg: ImplExecMsg = from_binary(&serialized_msg).unwrap();

        assert_eq!(serialized_msg, original_msg);
    }

    #[test]
    fn slice_deserialize_exec() {
        let deserialized: ImplExecMsg =
            from_slice(br#"{"remove_member": {"member": "some_member"}}"#).unwrap();
        assert_eq!(
            deserialized,
            ImplExecMsg::RemoveMember {
                member: "some_member".to_owned()
            }
        );
    }

    #[test]
    fn binary_serialize_query() {
        let original_msg = ImplQueryMsg::Query {
            member: "some_member".to_owned(),
        };

        let serialized_msg = to_binary(&original_msg).unwrap();
        let serialized_msg: ImplQueryMsg = from_binary(&serialized_msg).unwrap();

        assert_eq!(serialized_msg, original_msg);
    }

    #[test]
    fn slice_deserialize_query() {
        let deserialized: ImplQueryMsg =
            from_slice(br#"{"query": {"member": "some_member"}}"#).unwrap();
        assert_eq!(
            deserialized,
            ImplQueryMsg::Query {
                member: "some_member".to_owned()
            }
        );
    }
}

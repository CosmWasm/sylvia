use crate::error::ContractError;
use cw2::set_contract_version;
use cw_storage_plus::{Item, Map};
use sylvia::contract;
use sylvia::ctx::InstantiateCtx;
use sylvia::cw_std::{Addr, Deps, Empty, Response};
use sylvia::types::{CustomMsg, CustomQuery};

#[cfg(not(feature = "library"))]
use sylvia::entry_points;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Cw1WhitelistContract<E, Q> {
    pub(crate) admins: Map<&'static Addr, Empty>,
    pub(crate) mutable: Item<bool>,
    pub(crate) _phantom: std::marker::PhantomData<(E, Q)>,
}

#[cfg_attr(not(feature = "library"), entry_points(generics<Empty, Empty>))]
#[contract]
#[sv::error(ContractError)]
#[sv::messages(cw1 as Cw1)]
#[sv::messages(whitelist as Whitelist)]
#[sv::custom(msg=E, query=Q)]
impl<E, Q> Cw1WhitelistContract<E, Q>
where
    E: CustomMsg + 'static,
    Q: CustomQuery + 'static,
{
    pub const fn new() -> Self {
        Self {
            admins: Map::new("admins"),
            mutable: Item::new("mutable"),
            _phantom: std::marker::PhantomData,
        }
    }
    #[sv::msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: InstantiateCtx<Q>,
        admins: Vec<String>,
        mutable: bool,
    ) -> Result<Response<E>, ContractError> {
        set_contract_version(ctx.deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        for admin in admins {
            let admin = ctx.deps.api.addr_validate(&admin)?;
            self.admins.save(ctx.deps.storage, &admin, &Empty {})?;
        }

        self.mutable.save(ctx.deps.storage, &mutable)?;

        Ok(Response::new())
    }

    pub fn is_admin(&self, deps: Deps<Q>, addr: &Addr) -> bool {
        self.admins.has(deps.storage, addr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cw1::Cw1;
    use sylvia::cw_multi_test::{IntoAddr, IntoBech32};
    use sylvia::cw_std::testing::{message_info, mock_dependencies, mock_env};
    use sylvia::cw_std::{
        coin, coins, to_json_binary, BankMsg, CosmosMsg, StakingMsg, SubMsg, WasmMsg,
    };
    use whitelist::responses::AdminListResponse;
    use whitelist::Whitelist;

    #[test]
    fn instantiate_and_modify_config() {
        let mut deps = mock_dependencies();

        let alice = "alice".into_bech32();
        let bob = "bob".into_bech32();
        let carl = "carl".into_bech32();
        let mut admins = vec![alice.to_string(), bob.to_string(), carl.to_string()];
        admins.sort();

        let anyone = "anyone".into_addr();

        let contract: Cw1WhitelistContract<Empty, Empty> = Cw1WhitelistContract::new();

        // instantiate the contract
        let info = message_info(&anyone, &[]);
        contract
            .instantiate(
                (deps.as_mut(), mock_env(), info).into(),
                vec![alice.to_string(), bob.to_string(), carl.to_string()],
                true,
            )
            .unwrap();

        // ensure expected config
        let expected = AdminListResponse {
            admins,
            mutable: true,
        };
        assert_eq!(
            contract
                .admin_list((deps.as_ref(), mock_env()).into())
                .unwrap(),
            expected
        );

        // anyone cannot modify the contract
        let info = message_info(&anyone, &[]);
        let err = contract
            .update_admins(
                (deps.as_mut(), mock_env(), info).into(),
                vec![anyone.to_string()],
            )
            .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized);

        // but alice can kick out carl
        let info = message_info(&alice, &[]);
        contract
            .update_admins(
                (deps.as_mut(), mock_env(), info).into(),
                vec![alice.to_string(), bob.to_string()],
            )
            .unwrap();

        // ensure expected config
        let expected = AdminListResponse {
            admins: vec![alice.to_string(), bob.to_string()],
            mutable: true,
        };
        assert_eq!(
            contract
                .admin_list((deps.as_ref(), mock_env()).into())
                .unwrap(),
            expected
        );

        // carl cannot freeze it
        let info = message_info(&carl, &[]);
        let err = contract
            .freeze((deps.as_mut(), mock_env(), info).into())
            .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized);

        // but bob can
        let info = message_info(&bob, &[]);
        contract
            .freeze((deps.as_mut(), mock_env(), info).into())
            .unwrap();
        let expected = AdminListResponse {
            admins: vec![alice.to_string(), bob.to_string()],
            mutable: false,
        };
        assert_eq!(
            contract
                .admin_list((deps.as_ref(), mock_env()).into())
                .unwrap(),
            expected
        );

        // and now alice cannot change it again
        let info = message_info(&alice, &[]);
        let err = contract
            .update_admins(
                (deps.as_mut(), mock_env(), info).into(),
                vec![alice.to_string()],
            )
            .unwrap_err();
        assert_eq!(err, ContractError::ContractFrozen);
    }

    #[test]
    fn execute_messages_has_proper_permissions() {
        let mut deps = mock_dependencies();

        let alice = "alice".into_bech32();
        let bob = "bob".into_bech32();
        let carl = "carl".into_bech32();

        let contract: Cw1WhitelistContract<Empty, Empty> = Cw1WhitelistContract::new();

        // instantiate the contract
        let info = message_info(&bob, &[]);
        contract
            .instantiate(
                (deps.as_mut(), mock_env(), info).into(),
                vec![alice.to_string(), carl.to_string()],
                false,
            )
            .unwrap();

        let freeze = whitelist::sv::ExecMsg::Freeze {};
        let msgs = vec![
            BankMsg::Send {
                to_address: bob.to_string(),
                amount: coins(10000, "DAI"),
            }
            .into(),
            WasmMsg::Execute {
                contract_addr: "some contract".into(),
                msg: to_json_binary(&freeze).unwrap(),
                funds: vec![],
            }
            .into(),
        ];

        // bob cannot execute them
        let info = message_info(&bob, &[]);
        let err = contract
            .execute((deps.as_mut(), mock_env(), info).into(), msgs.clone())
            .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized);

        // but carl can
        let info = message_info(&carl, &[]);
        let res = contract
            .execute((deps.as_mut(), mock_env(), info).into(), msgs.clone())
            .unwrap();
        assert_eq!(
            res.messages,
            msgs.into_iter().map(SubMsg::new).collect::<Vec<_>>()
        );
        assert_eq!(res.attributes, [("action", "execute")]);
    }

    #[test]
    fn can_execute_query_works() {
        let mut deps = mock_dependencies();

        let alice = "alice".into_bech32();
        let bob = "bob".into_bech32();

        let anyone = "anyone".into_bech32();

        let contract: Cw1WhitelistContract<Empty, Empty> = Cw1WhitelistContract::new();

        // instantiate the contract
        let info = message_info(&anyone, &[]);
        contract
            .instantiate(
                (deps.as_mut(), mock_env(), info).into(),
                vec![alice.to_string(), bob.to_string()],
                false,
            )
            .unwrap();

        // let us make some queries... different msg types by owner and by other
        let send_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: anyone.to_string(),
            amount: coins(12345, "ushell"),
        });
        let staking_msg = CosmosMsg::Staking(StakingMsg::Delegate {
            validator: anyone.to_string(),
            amount: coin(70000, "ureef"),
        });

        // owner can send
        let res = contract
            .can_execute(
                (deps.as_ref(), mock_env()).into(),
                alice.to_string(),
                send_msg.clone(),
            )
            .unwrap();
        assert!(res.can_execute);

        // owner can stake
        let res = contract
            .can_execute(
                (deps.as_ref(), mock_env()).into(),
                bob.to_string(),
                staking_msg.clone(),
            )
            .unwrap();
        assert!(res.can_execute);

        // anyone cannot send
        let res = contract
            .can_execute(
                (deps.as_ref(), mock_env()).into(),
                anyone.to_string(),
                send_msg,
            )
            .unwrap();
        assert!(!res.can_execute);

        // anyone cannot stake
        let res = contract
            .can_execute(
                (deps.as_ref(), mock_env()).into(),
                anyone.to_string(),
                staking_msg,
            )
            .unwrap();
        assert!(!res.can_execute);
    }

    mod msgs {
        use sylvia::cw_std::{from_json, to_json_binary, BankMsg, Empty};

        use crate::contract::sv::{ContractExecMsg, ContractQueryMsg};

        #[test]
        fn freeze() {
            let original = whitelist::sv::ExecMsg::Freeze {};
            let serialized = to_json_binary(&original).unwrap();
            let deserialized: ContractExecMsg<Empty, Empty> = from_json(serialized).unwrap();

            assert_eq!(ContractExecMsg::Whitelist(original), deserialized);

            let json = br#"{
                "freeze": {}
            }"#;
            let deserialized: ContractExecMsg<Empty, Empty> = from_json(json).unwrap();

            assert_eq!(
                ContractExecMsg::Whitelist(whitelist::sv::ExecMsg::Freeze {}),
                deserialized
            );
        }

        #[test]
        fn update_admins() {
            let original = whitelist::sv::ExecMsg::UpdateAdmins {
                admins: vec!["admin1".to_owned(), "admin2".to_owned()],
            };
            let serialized = to_json_binary(&original).unwrap();
            let deserialized: ContractExecMsg<Empty, Empty> = from_json(serialized).unwrap();

            assert_eq!(ContractExecMsg::Whitelist(original), deserialized);

            let json = br#"{
                "update_admins": {
                    "admins": ["admin1", "admin3"]
                }
            }"#;
            let deserialized: ContractExecMsg<Empty, Empty> = from_json(json).unwrap();

            assert_eq!(
                ContractExecMsg::Whitelist(whitelist::sv::ExecMsg::UpdateAdmins {
                    admins: vec!["admin1".to_owned(), "admin3".to_owned()]
                }),
                deserialized
            );
        }

        #[test]
        fn admin_list() {
            let original = whitelist::sv::QueryMsg::AdminList {};
            let serialized = to_json_binary(&original).unwrap();
            let deserialized: ContractQueryMsg<Empty, Empty> = from_json(serialized).unwrap();

            assert_eq!(ContractQueryMsg::Whitelist(original), deserialized);

            let json = br#"{
                "admin_list": {}
            }"#;
            let deserialized: ContractQueryMsg<Empty, Empty> = from_json(json).unwrap();

            assert_eq!(
                ContractQueryMsg::Whitelist(whitelist::sv::QueryMsg::AdminList {}),
                deserialized
            );
        }

        #[test]
        fn execute() {
            let original = cw1::sv::ExecMsg::Execute {
                msgs: vec![BankMsg::Send {
                    to_address: "admin1".to_owned(),
                    amount: vec![],
                }
                .into()],
            };
            let serialized = to_json_binary(&original).unwrap();
            let deserialized: ContractExecMsg<Empty, Empty> = from_json(serialized).unwrap();
            assert_eq!(ContractExecMsg::Cw1(original), deserialized);
        }

        #[test]
        fn can_execute() {
            let original = cw1::sv::QueryMsg::CanExecute {
                sender: "admin".to_owned(),
                msg: BankMsg::Send {
                    to_address: "admin1".to_owned(),
                    amount: vec![],
                }
                .into(),
            };
            let serialized = to_json_binary(&original).unwrap();
            let deserialized: ContractQueryMsg<Empty, Empty> = from_json(serialized).unwrap();
            assert_eq!(ContractQueryMsg::Cw1(original), deserialized);
        }
    }
}

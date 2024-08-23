use serde::{Deserialize, Serialize};
use sylvia::cw_std::{CosmosMsg, Response, StdError, StdResult};
use sylvia::types::{CustomMsg, CustomQuery, ExecCtx, QueryCtx};
use sylvia::{interface, schemars};

#[derive(
    Serialize, Deserialize, Clone, PartialEq, Eq, sylvia::schemars::JsonSchema, Debug, Default,
)]
pub struct CanExecuteResp {
    pub can_execute: bool,
}

#[interface]
pub trait Cw1 {
    type Error: From<StdError>;
    type ExecC: CustomMsg;
    type QueryC: CustomQuery;

    /// Execute requests the contract to re-dispatch all these messages with the
    /// contract's address as sender. Every implementation has it's own logic to
    /// determine in
    #[sv::msg(exec)]
    fn execute(
        &self,
        ctx: ExecCtx<Self::QueryC>,
        msgs: Vec<CosmosMsg<Self::ExecC>>,
    ) -> Result<Response<Self::ExecC>, Self::Error>;

    /// Checks permissions of the caller on this proxy.
    /// If CanExecute returns true then a call to `Execute` with the same message,
    /// from the given sender, before any further state changes, should also succeed.
    #[sv::msg(query)]
    fn can_execute(
        &self,
        ctx: QueryCtx<Self::QueryC>,
        sender: String,
        msg: CosmosMsg<Self::ExecC>,
    ) -> StdResult<CanExecuteResp>;
}

#[cfg(test)]
mod tests {
    use sylvia::cw_std::{coins, from_json, to_json_binary, BankMsg, Empty};

    use crate::sv::{Cw1ExecMsg, Cw1QueryMsg};

    #[test]
    fn execute() {
        let original: Cw1ExecMsg<Empty> = super::sv::ExecMsg::Execute {
            msgs: vec![BankMsg::Send {
                to_address: "receiver".to_owned(),
                amount: coins(10, "atom"),
            }
            .into()],
        };

        let serialized = to_json_binary(&original).unwrap();
        let deserialized = from_json(serialized).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn execute_from_json() {
        let deserialized: Cw1ExecMsg<Empty> = from_json(br#"{"execute": { "msgs": [] }}"#).unwrap();
        assert_eq!(super::sv::ExecMsg::Execute { msgs: vec![] }, deserialized);
    }

    #[test]
    fn query() {
        let original: Cw1QueryMsg<Empty> = super::sv::QueryMsg::CanExecute {
            sender: "sender".to_owned(),
            msg: BankMsg::Send {
                to_address: "receiver".to_owned(),
                amount: coins(10, "atom"),
            }
            .into(),
        };

        let serialized = to_json_binary(&original).unwrap();
        let deserialized = from_json(serialized).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn query_from_json() {
        let deserialized: Cw1QueryMsg<Empty> = from_json(
            br#"{"can_execute": {
                "sender": "address",
                "msg": {
                    "bank": {
                        "send": {
                            "to_address": "receiver",
                            "amount": [
                                {
                                    "amount": "10",
                                    "denom": "atom"
                                }
                            ]
                        }
                    }
                }
            }}"#,
        )
        .unwrap();
        assert_eq!(
            super::sv::QueryMsg::CanExecute {
                sender: "address".to_owned(),
                msg: BankMsg::Send {
                    to_address: "receiver".to_owned(),
                    amount: coins(10, "atom"),
                }
                .into()
            },
            deserialized
        );
    }
}

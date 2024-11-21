use cosmwasm_schema::cw_serde;
use cosmwasm_std::to_json_binary;
use cw_storage_plus::Item;
use noop_contract::sv::{Executor, NoopContractInstantiateBuilder};
use sv::SubMsgMethods;
use sylvia::builder::instantiate::InstantiateBuilder;
use sylvia::ctx::{ExecCtx, InstantiateCtx, ReplyCtx};
use sylvia::cw_std::{Addr, Binary, Response, StdError, SubMsg};
use sylvia::cw_utils::{MsgInstantiateContractResponse, ParseReplyError};
use sylvia::types::Remote;
use sylvia::{contract, entry_points};
use thiserror::Error;

#[cw_serde]
pub struct ComplexData {
    pub message: String,
    pub number: u64,
}

#[allow(dead_code)]
mod noop_contract {
    use cosmwasm_std::{to_json_binary, Binary, StdResult};
    use sylvia::ctx::{ExecCtx, InstantiateCtx};
    use sylvia::{contract, entry_points};

    use sylvia::cw_std::Response;

    use crate::ComplexData;

    pub struct NoopContract;

    #[entry_points]
    #[contract]
    impl NoopContract {
        pub const fn new() -> Self {
            Self
        }

        #[sv::msg(instantiate)]
        fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(exec)]
        fn noop(&self, _ctx: ExecCtx, data: Option<Binary>) -> StdResult<Response> {
            let resp = match data {
                Some(data) => Response::new().set_data(data),
                None => Response::new(),
            };

            Ok(resp)
        }

        #[sv::msg(exec)]
        fn noop_complex_data(&self, _ctx: ExecCtx) -> StdResult<Response> {
            let data = ComplexData {
                message: "Hello".to_string(),
                number: 42,
            };
            Ok(Response::new().set_data(to_json_binary(&data)?))
        }
    }
}

#[cw_serde]
pub struct InstantiatePayload {
    pub sender: Addr,
}

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    ParseReply(#[from] ParseReplyError),
}

pub struct Contract {
    remote: Item<Remote<'static, noop_contract::NoopContract>>,
}

#[entry_points]
#[contract]
#[sv::error(ContractError)]
#[sv::features(replies)]
impl Contract {
    pub fn new() -> Self {
        Self {
            remote: Item::new("remote"),
        }
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: InstantiateCtx,
        remote_code_id: u64,
    ) -> Result<Response, ContractError> {
        // Custom type can be used as a payload.
        let payload = InstantiatePayload {
            sender: ctx.info.sender,
        };
        let sub_msg = InstantiateBuilder::noop_contract(remote_code_id)?
            .with_label("noop")
            .build()
            .remote_instantiated(payload)?;

        Ok(Response::new().add_submessage(sub_msg))
    }

    #[sv::msg(exec)]
    fn send_message_expecting_data(
        &self,
        ctx: ExecCtx,
        data: Option<Binary>,
        reply_id: u64,
    ) -> Result<Response, ContractError> {
        let msg = self
            .remote
            .load(ctx.deps.storage)?
            .executor()
            .noop(data)?
            .build();
        let submsg = SubMsg::reply_on_success(msg, reply_id);

        Ok(Response::new().add_submessage(submsg))
    }

    #[sv::msg(exec)]
    fn send_message_expecting_complex_data(&self, ctx: ExecCtx) -> Result<Response, ContractError> {
        let submsg = self
            .remote
            .load(ctx.deps.storage)?
            .executor()
            .noop_complex_data()?
            .build()
            .complex_data(Binary::default())?;

        Ok(Response::new().add_submessage(submsg))
    }

    #[sv::msg(reply, reply_on=success)]
    fn remote_instantiated(
        &self,
        ctx: ReplyCtx,
        #[sv::data(instantiate)] data: MsgInstantiateContractResponse,
        _instantiate_payload: InstantiatePayload,
    ) -> Result<Response, ContractError> {
        let remote_addr = Addr::unchecked(data.contract_address);

        self.remote
            .save(ctx.deps.storage, &Remote::new(remote_addr))?;

        Ok(Response::new())
    }

    #[sv::msg(reply, reply_on=success)]
    fn _optional_remote_instantiated(
        &self,
        _ctx: ReplyCtx,
        #[sv::data(instantiate, opt)] _data: Option<MsgInstantiateContractResponse>,
        #[sv::payload(raw)] _payload: Binary,
    ) -> Result<Response, ContractError> {
        Ok(Response::new())
    }

    #[sv::msg(reply, reply_on=success)]
    fn data_raw_opt(
        &self,
        _ctx: ReplyCtx,
        #[sv::data(raw, opt)] _data: Option<Binary>,
        #[sv::payload(raw)] _payload: Binary,
    ) -> Result<Response, ContractError> {
        Ok(Response::new())
    }

    #[sv::msg(reply, reply_on=success)]
    fn data_raw(
        &self,
        _ctx: ReplyCtx,
        #[sv::data(raw)] _data: Binary,
        #[sv::payload(raw)] _payload: Binary,
    ) -> Result<Response, ContractError> {
        Ok(Response::new())
    }

    #[sv::msg(reply, reply_on=success)]
    fn data_opt(
        &self,
        _ctx: ReplyCtx,
        #[sv::data(opt)] _data: Option<String>,
        #[sv::payload(raw)] _payload: Binary,
    ) -> Result<Response, ContractError> {
        Ok(Response::new())
    }

    #[sv::msg(reply, reply_on=success)]
    fn data(
        &self,
        _ctx: ReplyCtx,
        #[sv::data] _data: String,
        #[sv::payload(raw)] _payload: Binary,
    ) -> Result<Response, ContractError> {
        Ok(Response::new())
    }

    #[sv::msg(reply, reply_on=success)]
    fn complex_data(
        &self,
        _ctx: ReplyCtx,
        #[sv::data] _data: ComplexData,
        #[sv::payload(raw)] _payload: Binary,
    ) -> Result<Response, ContractError> {
        Ok(Response::new())
    }

    #[sv::msg(reply, reply_on=success)]
    fn no_data(
        &self,
        _ctx: ReplyCtx,
        #[sv::payload(raw)] _payload: Binary,
    ) -> Result<Response, ContractError> {
        Ok(Response::new())
    }
}

use crate::noop_contract::sv::mt::CodeId as NoopCodeId;
use crate::sv::mt::{CodeId, ContractProxy};
use crate::sv::{DATA_OPT_REPLY_ID, DATA_RAW_OPT_REPLY_ID, DATA_RAW_REPLY_ID, DATA_REPLY_ID};

use sylvia::cw_multi_test::IntoBech32;
use sylvia::multitest::App;

#[test]
fn data_instantiate() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);
    let noop_code_id = NoopCodeId::store_code(&app);

    let owner = "owner".into_bech32();

    // Trigger remote instantiation reply
    let _ = code_id
        .instantiate(noop_code_id.code_id())
        .with_label("Contract")
        .call(&owner)
        .unwrap();
}

#[test]
fn data_raw_opt() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);
    let noop_code_id = NoopCodeId::store_code(&app);

    let owner = "owner".into_bech32();
    let data = Some(to_json_binary(&String::from("some_data")).unwrap());

    // Trigger remote instantiation reply
    let contract = code_id
        .instantiate(noop_code_id.code_id())
        .with_label("Contract")
        .call(&owner)
        .unwrap();

    // Should forward `data` in every case
    contract
        .send_message_expecting_data(None, DATA_RAW_OPT_REPLY_ID)
        .call(&owner)
        .unwrap();

    contract
        .send_message_expecting_data(data.clone(), DATA_RAW_OPT_REPLY_ID)
        .call(&owner)
        .unwrap();
}

#[test]
fn data_raw() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);
    let noop_code_id = NoopCodeId::store_code(&app);

    let owner = "owner".into_bech32();
    let data = Some(to_json_binary(&String::from("some_data")).unwrap());

    // Trigger remote instantiation reply
    let contract = code_id
        .instantiate(noop_code_id.code_id())
        .with_label("Contract")
        .call(&owner)
        .unwrap();

    // Should forward `data` if `Some` and return error if `None`
    let err = contract
        .send_message_expecting_data(None, DATA_RAW_REPLY_ID)
        .call(&owner)
        .unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Missing reply data field.").into()
    );

    contract
        .send_message_expecting_data(data.clone(), DATA_RAW_REPLY_ID)
        .call(&owner)
        .unwrap();
}

#[test]
fn data_opt() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);
    let noop_code_id = NoopCodeId::store_code(&app);

    let owner = "owner".into_bech32();
    let data = Some(to_json_binary(&String::from("some_data")).unwrap());
    let invalid_data = Some(Binary::from("InvalidData".as_bytes()));

    // Trigger remote instantiation reply
    let contract = code_id
        .instantiate(noop_code_id.code_id())
        .with_label("Contract")
        .call(&owner)
        .unwrap();

    // Should forward deserialized `data` if `Some` or None and return error if deserialization fails
    contract
        .send_message_expecting_data(None, DATA_OPT_REPLY_ID)
        .call(&owner)
        .unwrap();

    let err = contract
        .send_message_expecting_data(invalid_data.clone(), DATA_OPT_REPLY_ID)
        .call(&owner)
        .unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err(
            "Invalid reply data at block height: 12345, transaction id: 0.\nSerde error while deserializing Error parsing into type alloc::string::String: Invalid type"
        ).into()
    );

    contract
        .send_message_expecting_data(data.clone(), DATA_OPT_REPLY_ID)
        .call(&owner)
        .unwrap();
}

#[test]
fn data() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);
    let noop_code_id = NoopCodeId::store_code(&app);

    let owner = "owner".into_bech32();
    let data = Some(to_json_binary(&String::from("some_data")).unwrap());
    let invalid_data = Some(Binary::from("InvalidData".as_bytes()));

    // Trigger remote instantiation reply
    let contract = code_id
        .instantiate(noop_code_id.code_id())
        .with_label("Contract")
        .call(&owner)
        .unwrap();

    // Should forward deserialized `data` if `Some` and return error if `None` or if deserialization fails
    let err = contract
        .send_message_expecting_data(None, DATA_REPLY_ID)
        .call(&owner)
        .unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Missing reply data field.").into()
    );

    let err = contract
        .send_message_expecting_data(invalid_data, DATA_REPLY_ID)
        .call(&owner)
        .unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err(
            "Invalid reply data at block height: 12345, transaction id: 0.\nSerde error while deserializing Error parsing into type alloc::string::String: Invalid type"
        ).into()
    );

    contract
        .send_message_expecting_data(data, DATA_REPLY_ID)
        .call(&owner)
        .unwrap();

    // Should deserialize custom data types
    contract
        .send_message_expecting_complex_data()
        .call(&owner)
        .unwrap();
}

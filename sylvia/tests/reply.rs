#![cfg(feature = "sv_replies")]

use cosmwasm_std::SubMsgResult;
use cw_storage_plus::Item;
use cw_utils::{parse_instantiate_response_data, ParseReplyError};
use noop_contract::sv::Executor;
use sv::{ALWAYS_REPLY_ID, FAILURE_REPLY_ID, REMOTE_INSTANTIATED_REPLY_ID, SUCCESS_REPLY_ID};
use sylvia::cw_std::{to_json_binary, Addr, Binary, ReplyOn, Response, StdError, SubMsg, WasmMsg};
use sylvia::types::{ExecCtx, InstantiateCtx, QueryCtx, Remote, ReplyCtx};
use sylvia::{contract, entry_points};
use thiserror::Error;

#[allow(dead_code)]
mod noop_contract {
    use cosmwasm_std::{StdError, StdResult};
    use sylvia::types::{ExecCtx, InstantiateCtx};
    use sylvia::{contract, entry_points};

    use sylvia::cw_std::Response;

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
        fn noop(&self, _ctx: ExecCtx, should_fail: bool) -> StdResult<Response> {
            if should_fail {
                Err(StdError::generic_err("Failed as requested"))
            } else {
                Ok(Response::new())
            }
        }
    }
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
    last_reply: Item<u64>,
}

#[entry_points]
#[contract]
#[sv::error(ContractError)]
impl Contract {
    pub fn new() -> Self {
        Self {
            remote: Item::new("remote"),
            last_reply: Item::new("last_reply"),
        }
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(
        &self,
        _ctx: InstantiateCtx,
        remote_code_id: u64,
    ) -> Result<Response, ContractError> {
        let msg = WasmMsg::Instantiate {
            admin: None,
            code_id: remote_code_id,
            msg: to_json_binary(&noop_contract::sv::InstantiateMsg::new())?,
            funds: vec![],
            label: "noop".to_string(),
        };
        let sub_msg = SubMsg::reply_on_success(msg, REMOTE_INSTANTIATED_REPLY_ID);
        Ok(Response::new().add_submessage(sub_msg))
    }

    #[sv::msg(exec)]
    pub fn call_remote(
        &self,
        ctx: ExecCtx,
        reply_id: u64,
        should_fail: bool,
        reply_on: ReplyOn,
    ) -> Result<Response, ContractError> {
        let msg = self
            .remote
            .load(ctx.deps.storage)?
            .executor()
            .noop(should_fail)?
            .build();
        let mut msg = SubMsg::new(msg);
        msg.id = reply_id;
        msg.reply_on = reply_on;

        Ok(Response::new().add_submessage(msg))
    }

    #[sv::msg(query)]
    pub fn last_reply(&self, ctx: QueryCtx) -> Result<u64, ContractError> {
        self.last_reply.load(ctx.deps.storage).map_err(Into::into)
    }

    #[sv::msg(reply, reply_on=success)]
    fn remote_instantiated(
        &self,
        ctx: ReplyCtx,
        data: Option<Binary>,
        _payload: Binary,
    ) -> Result<Response, ContractError> {
        self.last_reply
            .save(ctx.deps.storage, &REMOTE_INSTANTIATED_REPLY_ID)?;
        let init_data = parse_instantiate_response_data(&data.unwrap())?;
        let remote_addr = Addr::unchecked(init_data.contract_address);

        self.remote
            .save(ctx.deps.storage, &Remote::new(remote_addr))?;

        Ok(Response::new())
    }

    #[sv::msg(reply, handlers=[success, both], reply_on=success)]
    fn success(
        &self,
        ctx: ReplyCtx,
        _data: Option<Binary>,
        _payload: Binary,
    ) -> Result<Response, ContractError> {
        self.last_reply.save(ctx.deps.storage, &SUCCESS_REPLY_ID)?;

        Ok(Response::new())
    }

    #[sv::msg(reply, handlers=[failure, both], reply_on=failure)]
    fn failure(
        &self,
        ctx: ReplyCtx,
        _error: String,
        _payload: Binary,
    ) -> Result<Response, ContractError> {
        self.last_reply.save(ctx.deps.storage, &FAILURE_REPLY_ID)?;

        Ok(Response::new())
    }

    #[sv::msg(reply, reply_on=always)]
    fn always(
        &self,
        ctx: ReplyCtx,
        _result: SubMsgResult,
        _payload: Binary,
    ) -> Result<Response, ContractError> {
        self.last_reply.save(ctx.deps.storage, &ALWAYS_REPLY_ID)?;

        Ok(Response::new())
    }
}

mod tests {
    use crate::noop_contract::sv::mt::CodeId as NoopCodeId;
    use crate::sv::mt::{CodeId, ContractProxy};
    use crate::sv::{
        ALWAYS_REPLY_ID, BOTH_REPLY_ID, FAILURE_REPLY_ID, REMOTE_INSTANTIATED_REPLY_ID,
        SUCCESS_REPLY_ID,
    };

    use sylvia::cw_multi_test::IntoBech32;
    use sylvia::cw_std::{ReplyOn, StdError};
    use sylvia::multitest::App;

    #[test]
    fn dispatch_replies() {
        let app = App::default();
        let code_id = CodeId::store_code(&app);
        let noop_code_id = NoopCodeId::store_code(&app);

        let owner = "owner".into_bech32();

        // Trigger remote instantiation reply
        let contract = code_id
            .instantiate(noop_code_id.code_id())
            .with_label("Contract")
            .call(&owner)
            .unwrap();
        let last_reply = contract.last_reply().unwrap();
        assert_eq!(last_reply, REMOTE_INSTANTIATED_REPLY_ID);

        // Should not dispatch if expected success and execution failed
        let err = contract
            .call_remote(SUCCESS_REPLY_ID, true, ReplyOn::Success)
            .call(&owner)
            .unwrap_err();
        assert_eq!(err, StdError::generic_err("Failed as requested").into());
        let last_reply = contract.last_reply().unwrap();
        assert_eq!(last_reply, REMOTE_INSTANTIATED_REPLY_ID);

        // Should dispatch if expected success and execution succeeded
        contract
            .call_remote(SUCCESS_REPLY_ID, false, ReplyOn::Success)
            .call(&owner)
            .unwrap();
        let last_reply = contract.last_reply().unwrap();
        assert_eq!(last_reply, SUCCESS_REPLY_ID);

        // Should not dispatch if expected failure and execution succeeded
        contract
            .call_remote(FAILURE_REPLY_ID, false, ReplyOn::Error)
            .call(&owner)
            .unwrap();
        let last_reply = contract.last_reply().unwrap();
        assert_eq!(last_reply, SUCCESS_REPLY_ID);

        // Should dispatch if expected failure and execution failed
        contract
            .call_remote(FAILURE_REPLY_ID, true, ReplyOn::Error)
            .call(&owner)
            .unwrap();
        let last_reply = contract.last_reply().unwrap();
        assert_eq!(last_reply, FAILURE_REPLY_ID);

        // Should dispatch if expected any result and execution succeeded
        contract
            .call_remote(ALWAYS_REPLY_ID, false, ReplyOn::Always)
            .call(&owner)
            .unwrap();
        let last_reply = contract.last_reply().unwrap();
        assert_eq!(last_reply, ALWAYS_REPLY_ID);

        // Should dispatch if expected any result and execution failed
        contract
            .call_remote(ALWAYS_REPLY_ID, true, ReplyOn::Always)
            .call(&owner)
            .unwrap();
        let last_reply = contract.last_reply().unwrap();
        assert_eq!(last_reply, ALWAYS_REPLY_ID);

        // Should dispatch if expected both results and execution succeeded
        contract
            .call_remote(BOTH_REPLY_ID, false, ReplyOn::Success)
            .call(&owner)
            .unwrap();
        let last_reply = contract.last_reply().unwrap();
        assert_eq!(last_reply, SUCCESS_REPLY_ID);

        // Should dispatch if expected both results and execution failed
        contract
            .call_remote(BOTH_REPLY_ID, true, ReplyOn::Error)
            .call(&owner)
            .unwrap();
        let last_reply = contract.last_reply().unwrap();
        assert_eq!(last_reply, FAILURE_REPLY_ID);

        // Should return error if unknown reply ID received
        let unknown_reply_id = 42u64;
        let err = contract
            .call_remote(unknown_reply_id, false, ReplyOn::Success)
            .call(&owner)
            .unwrap_err();
        assert_eq!(
            err,
            StdError::generic_err(format!("Unknown reply id: {}.", unknown_reply_id)).into()
        );
    }
}

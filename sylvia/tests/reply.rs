use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, BankMsg, CosmosMsg, Empty, SubMsgResult};
use cw_storage_plus::Item;
use cw_utils::{parse_instantiate_response_data, ParseReplyError};
use noop_contract::sv::{Executor, NoopContractInstantiateBuilder};
use sv::{
    SubMsgMethods, ALWAYS_REPLY_ID, FAILURE_REPLY_ID, REMOTE_INSTANTIATED_REPLY_ID,
    SUCCESS_REPLY_ID,
};
use sylvia::builder::instantiate::InstantiateBuilder;
use sylvia::cw_std::{Addr, Binary, Response, StdError, SubMsg};
use sylvia::types::{CustomMsg, CustomQuery, ExecCtx, InstantiateCtx, QueryCtx, Remote, ReplyCtx};
use sylvia::{contract, entry_points};
use thiserror::Error;

#[allow(dead_code)]
mod noop_contract {
    use cosmwasm_std::{Empty, StdError, StdResult};
    use sylvia::types::{CustomMsg, CustomQuery, ExecCtx, InstantiateCtx};
    use sylvia::{contract, entry_points};

    use sylvia::cw_std::Response;

    pub struct NoopContract<M, Q> {
        _phantom: std::marker::PhantomData<(M, Q)>,
    }

    #[entry_points(generics<Empty, Empty>)]
    #[contract]
    #[sv::custom(msg=M, query=Q)]
    impl<M, Q> NoopContract<M, Q>
    where
        M: CustomMsg + 'static,
        Q: CustomQuery + 'static,
    {
        pub const fn new() -> Self {
            Self {
                _phantom: std::marker::PhantomData,
            }
        }

        #[sv::msg(instantiate)]
        fn instantiate(&self, _ctx: InstantiateCtx<Q>) -> StdResult<Response<M>> {
            Ok(Response::new())
        }

        #[sv::msg(exec)]
        fn noop(&self, _ctx: ExecCtx<Q>, should_fail: bool) -> StdResult<Response<M>> {
            if should_fail {
                Err(StdError::generic_err("Failed as requested"))
            } else {
                Ok(Response::new())
            }
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

pub struct Contract<M, Q> {
    remote: Item<Remote<'static, noop_contract::NoopContract<Empty, Empty>>>,
    last_reply: Item<u64>,
    _phantom: std::marker::PhantomData<(M, Q)>,
}

#[entry_points(generics<Empty, Empty>)]
#[contract]
#[sv::error(ContractError)]
#[sv::custom(msg=M, query=Q)]
#[sv::features(replies)]
impl<M, Q> Contract<M, Q>
where
    M: CustomMsg + 'static,
    Q: CustomQuery + 'static,
{
    pub fn new() -> Self {
        Self {
            remote: Item::new("remote"),
            last_reply: Item::new("last_reply"),
            _phantom: std::marker::PhantomData,
        }
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: InstantiateCtx<Q>,
        remote_code_id: u64,
    ) -> Result<Response<M>, ContractError> {
        // Custom type can be used as a payload.
        let payload = InstantiatePayload {
            sender: ctx.info.sender,
        };
        let sub_msg = InstantiateBuilder::noop_contract(remote_code_id)?
            .with_label("noop")
            .build()
            .remote_instantiated(to_json_binary(&payload)?)?;
        // TODO: Blocked by https://github.com/CosmWasm/cw-multi-test/pull/216. Uncomment when new
        // MultiTest version is released.
        // Payload is not currently forwarded in the MultiTest.
        // .remote_instantiated(payload)?;

        Ok(Response::new().add_submessage(sub_msg))
    }

    #[sv::msg(exec)]
    pub fn call_remote_success(
        &self,
        ctx: ExecCtx<Q>,
        should_fail: bool,
    ) -> Result<Response<M>, ContractError> {
        let msg = self
            .remote
            .load(ctx.deps.storage)?
            .executor()
            .noop(should_fail)?
            .build()
            .success(Binary::default())?;

        Ok(Response::new().add_submessage(msg))
    }

    #[sv::msg(exec)]
    pub fn call_remote_failure(
        &self,
        ctx: ExecCtx<Q>,
        should_fail: bool,
    ) -> Result<Response<M>, ContractError> {
        let msg = self
            .remote
            .load(ctx.deps.storage)?
            .executor()
            .noop(should_fail)?
            .build()
            .failure(Binary::default())?;

        Ok(Response::new().add_submessage(msg))
    }

    #[sv::msg(exec)]
    pub fn call_remote_both(
        &self,
        ctx: ExecCtx<Q>,
        should_fail: bool,
    ) -> Result<Response<M>, ContractError> {
        let msg = self
            .remote
            .load(ctx.deps.storage)?
            .executor()
            .noop(should_fail)?
            .build()
            .both(Binary::default())?;

        Ok(Response::new().add_submessage(msg))
    }

    #[sv::msg(exec)]
    pub fn call_remote_always(
        &self,
        ctx: ExecCtx<Q>,
        should_fail: bool,
    ) -> Result<Response<M>, ContractError> {
        // Tuple can be used as a payload.
        let payload = to_json_binary(&(42_u32, "Hello, world!".to_string()))?;

        let msg = self
            .remote
            .load(ctx.deps.storage)?
            .executor()
            .noop(should_fail)?
            .build()
            .always(payload)?;

        Ok(Response::new().add_submessage(msg))
    }

    #[sv::msg(exec)]
    pub fn call_remote_unknown_id(
        &self,
        ctx: ExecCtx<Q>,
        should_fail: bool,
        reply_id: u64,
    ) -> Result<Response<M>, ContractError> {
        let msg = self
            .remote
            .load(ctx.deps.storage)?
            .executor()
            .noop(should_fail)?
            .build();

        let msg = SubMsg::reply_always(msg, reply_id);

        Ok(Response::new().add_submessage(msg))
    }

    #[sv::msg(query)]
    pub fn last_reply(&self, ctx: QueryCtx<Q>) -> Result<u64, ContractError> {
        self.last_reply.load(ctx.deps.storage).map_err(Into::into)
    }

    #[sv::msg(reply, reply_on=success)]
    fn remote_instantiated(
        &self,
        ctx: ReplyCtx<Q>,
        data: Option<Binary>,
        // TODO: Blocked by https://github.com/CosmWasm/cw-multi-test/pull/216. Uncomment when new
        // MultiTest version is released.
        // Payload is not currently forwarded in the MultiTest.
        // _instantiate_payload: InstantiatePayload,
        #[sv::payload] _payload: Binary,
    ) -> Result<Response<M>, ContractError> {
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
        ctx: ReplyCtx<Q>,
        _data: Option<Binary>,
        #[sv::payload] _payload: Binary,
    ) -> Result<Response<M>, ContractError> {
        self.last_reply.save(ctx.deps.storage, &SUCCESS_REPLY_ID)?;

        Ok(Response::new())
    }

    #[sv::msg(reply, handlers=[failure, both], reply_on=failure)]
    fn failure(
        &self,
        ctx: ReplyCtx<Q>,
        _error: String,
        #[sv::payload] _payload: Binary,
    ) -> Result<Response<M>, ContractError> {
        self.last_reply.save(ctx.deps.storage, &FAILURE_REPLY_ID)?;

        Ok(Response::new())
    }

    #[sv::msg(reply, reply_on=always)]
    fn always(
        &self,
        ctx: ReplyCtx<Q>,
        _result: SubMsgResult,
        #[sv::payload] _payload: Binary,
        // _first_part_payload: u32,
        // _second_part_payload: String,
    ) -> Result<Response<M>, ContractError> {
        self.last_reply.save(ctx.deps.storage, &ALWAYS_REPLY_ID)?;

        Ok(Response::new())
    }

    #[sv::msg(exec)]
    fn send_cosmos_messages(&self, ctx: ExecCtx<Q>) -> Result<Response<M>, ContractError> {
        let remote_addr = self.remote.load(ctx.deps.storage)?;
        let cosmos_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: remote_addr.as_ref().to_string(),
            amount: vec![],
        });
        let submsg = cosmos_msg.always(Binary::default())?;
        Ok(Response::new().add_submessage(submsg))
    }
}

mod tests {
    use crate::noop_contract::sv::mt::CodeId as NoopCodeId;
    use crate::sv::mt::{CodeId, ContractProxy};
    use crate::sv::{
        ALWAYS_REPLY_ID, FAILURE_REPLY_ID, REMOTE_INSTANTIATED_REPLY_ID, SUCCESS_REPLY_ID,
    };

    use sylvia::cw_multi_test::IntoBech32;
    use sylvia::cw_std::StdError;
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
        let err = contract.call_remote_success(true).call(&owner).unwrap_err();
        assert_eq!(err, StdError::generic_err("Failed as requested").into());
        let last_reply = contract.last_reply().unwrap();
        assert_eq!(last_reply, REMOTE_INSTANTIATED_REPLY_ID);

        // Should dispatch if expected success and execution succeeded
        contract.call_remote_success(false).call(&owner).unwrap();
        let last_reply = contract.last_reply().unwrap();
        assert_eq!(last_reply, SUCCESS_REPLY_ID);

        // Should not dispatch if expected failure and execution succeeded
        contract.call_remote_failure(false).call(&owner).unwrap();
        let last_reply = contract.last_reply().unwrap();
        assert_eq!(last_reply, SUCCESS_REPLY_ID);

        // Should dispatch if expected failure and execution failed
        contract.call_remote_failure(true).call(&owner).unwrap();
        let last_reply = contract.last_reply().unwrap();
        assert_eq!(last_reply, FAILURE_REPLY_ID);

        // Should dispatch if expected any result and execution succeeded
        contract.call_remote_always(false).call(&owner).unwrap();
        let last_reply = contract.last_reply().unwrap();
        assert_eq!(last_reply, ALWAYS_REPLY_ID);

        // Should dispatch if expected both results and execution succeeded
        contract.call_remote_both(false).call(&owner).unwrap();
        let last_reply = contract.last_reply().unwrap();
        assert_eq!(last_reply, SUCCESS_REPLY_ID);

        // Should dispatch if expected any result and execution failed
        contract.call_remote_always(true).call(&owner).unwrap();
        let last_reply = contract.last_reply().unwrap();
        assert_eq!(last_reply, ALWAYS_REPLY_ID);

        // Should dispatch if expected both results and execution failed
        contract.call_remote_both(true).call(&owner).unwrap();
        let last_reply = contract.last_reply().unwrap();
        assert_eq!(last_reply, FAILURE_REPLY_ID);

        // Should send the cosmos message
        contract.send_cosmos_messages().call(&owner).unwrap();
        let last_reply = contract.last_reply().unwrap();
        assert_eq!(last_reply, ALWAYS_REPLY_ID);

        // Should return error if unknown reply ID received
        let unknown_reply_id = 42u64;
        let err = contract
            .call_remote_unknown_id(false, unknown_reply_id)
            .call(&owner)
            .unwrap_err();
        assert_eq!(
            err,
            StdError::generic_err(format!("Unknown reply id: {}.", unknown_reply_id)).into()
        );
    }
}

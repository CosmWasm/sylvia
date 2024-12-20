#[cfg(all(test, feature = "mt"))]
use cw_multi_test::IntoBech32;
use sylvia::cw_std::testing::{mock_dependencies, mock_env};
use sylvia::cw_std::{from_json, Reply, SubMsgResponse, SubMsgResult};

#[allow(dead_code)]
mod noop_contract {
    use sylvia::ctx::{ExecCtx, InstantiateCtx};
    use sylvia::{contract, entry_points};

    use sylvia::cw_std::{Response, StdResult};

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
        fn noop(&self, _ctx: ExecCtx) -> StdResult<Response> {
            Ok(Response::new())
        }
    }
}

mod reply_contract {
    use cosmwasm_std::Reply;
    use sylvia::ctx::{ExecCtx, InstantiateCtx};
    #[allow(deprecated)]
    use sylvia::types::ReplyCtx;
    use sylvia::{contract, entry_points};

    use sylvia::cw_std::{to_json_binary, Response, StdResult, SubMsg, WasmMsg};

    use super::noop_contract;

    pub struct ReplyContract;

    #[allow(dead_code)]
    #[entry_points]
    #[contract]
    impl ReplyContract {
        pub const fn new() -> Self {
            Self
        }

        #[sv::msg(instantiate)]
        fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[sv::msg(exec)]
        fn poke(&self, _ctx: ExecCtx, noop: String) -> StdResult<Response> {
            let msg = noop_contract::sv::ExecMsg::Noop {};
            let msg = WasmMsg::Execute {
                contract_addr: noop,
                msg: to_json_binary(&msg)?,
                funds: vec![],
            };
            let msg = SubMsg::reply_always(msg, 1);

            let resp = Response::new().add_submessage(msg);
            Ok(resp)
        }

        #[sv::msg(reply)]
        #[allow(deprecated)]
        fn reply(&self, _ctx: ReplyCtx, _reply: Reply) -> StdResult<Response> {
            let resp = Response::new().set_data(to_json_binary("data")?);
            Ok(resp)
        }
    }
}

#[test]
fn entry_point_generation() {
    let msg = Reply {
        id: 0,
        payload: Default::default(),
        gas_used: 0,
        #[allow(deprecated)]
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: None,
            msg_responses: vec![],
        }),
    };
    let mut deps = mock_dependencies();
    let env = mock_env();

    let resp = reply_contract::entry_points::reply(deps.as_mut(), env, msg).unwrap();
    let data: String = from_json(resp.data.unwrap()).unwrap();

    assert_eq!(data, "data");
}

#[cfg(all(test, feature = "mt"))]
#[test]
fn mt_helper_generation() {
    use crate::reply_contract::sv::mt::ReplyContractProxy;
    let app = sylvia::multitest::App::default();
    let owner = "owner".into_bech32();

    let noop_contract_code = noop_contract::sv::mt::CodeId::store_code(&app);
    let noop_contract = noop_contract_code.instantiate().call(&owner).unwrap();

    let reply_contract_code = reply_contract::sv::mt::CodeId::store_code(&app);
    let reply_contract = reply_contract_code.instantiate().call(&owner).unwrap();

    let resp = reply_contract
        .poke(noop_contract.contract_addr.to_string())
        .call(&owner)
        .unwrap();

    let data: String = from_json(resp.data.unwrap()).unwrap();

    assert_eq!(data, "data");
}

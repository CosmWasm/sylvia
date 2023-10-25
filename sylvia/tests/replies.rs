use sylvia::cw_std::testing::{mock_dependencies, mock_env};
use sylvia::cw_std::{from_binary, Reply, SubMsgResponse, SubMsgResult};

#[allow(dead_code)]
mod noop_contract {
    use sylvia::contract;
    use sylvia::types::{ExecCtx, InstantiateCtx};

    use sylvia::cw_std::{Response, StdResult};

    pub struct NoopContract;

    #[cfg(not(tarpaulin_include))]
    #[contract]
    impl NoopContract {
        pub const fn new() -> Self {
            Self
        }

        #[msg(instantiate)]
        fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[msg(exec)]
        fn noop(&self, _ctx: ExecCtx) -> StdResult<Response> {
            Ok(Response::new())
        }
    }
}

mod reply_contract {
    use sylvia::types::{ExecCtx, InstantiateCtx, ReplyCtx};
    use sylvia::{contract, entry_points};

    use sylvia::cw_std::{to_binary, Reply, Response, StdResult, SubMsg, WasmMsg};

    use super::noop_contract;

    pub struct ReplyContract;

    #[allow(dead_code)]
    #[cfg(not(tarpaulin_include))]
    #[entry_points]
    #[contract]
    impl ReplyContract {
        pub const fn new() -> Self {
            Self
        }

        #[msg(instantiate)]
        fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
            Ok(Response::new())
        }

        #[msg(exec)]
        fn poke(&self, _ctx: ExecCtx, noop: String) -> StdResult<Response> {
            let msg = noop_contract::sv::ExecMsg::Noop {};
            let msg = WasmMsg::Execute {
                contract_addr: noop,
                msg: to_binary(&msg)?,
                funds: vec![],
            };
            let msg = SubMsg::reply_always(msg, 1);

            let resp = Response::new().add_submessage(msg);
            Ok(resp)
        }

        #[msg(reply)]
        fn reply(&self, _ctx: ReplyCtx, _msg: Reply) -> StdResult<Response> {
            let resp = Response::new().set_data(to_binary("data")?);
            Ok(resp)
        }
    }
}

#[test]
fn entry_point_generation() {
    let msg = Reply {
        id: 0,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: None,
        }),
    };
    let mut deps = mock_dependencies();
    let env = mock_env();

    let resp = reply_contract::entry_points::reply(deps.as_mut(), env, msg).unwrap();
    let data: String = from_binary(&resp.data.unwrap()).unwrap();

    assert_eq!(data, "data");
}

#[cfg(all(test, feature = "mt"))]
#[test]
fn mt_helper_generation() {
    let app = sylvia::multitest::App::default();
    let owner = "owner";

    let noop_contract_code = noop_contract::sv::multitest_utils::CodeId::store_code(&app);
    let noop_contract = noop_contract_code.instantiate().call(owner).unwrap();

    let reply_contract_code = reply_contract::sv::multitest_utils::CodeId::store_code(&app);
    let reply_contract = reply_contract_code.instantiate().call(owner).unwrap();

    let resp = reply_contract
        .poke(noop_contract.contract_addr.to_string())
        .call(owner)
        .unwrap();

    let data: String = from_binary(&resp.data.unwrap()).unwrap();

    assert_eq!(data, "data");
}

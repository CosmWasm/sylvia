use anyhow::{bail, Result as AnyResult};
use cosmwasm_std::{
    from_slice, to_binary, Addr, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError,
    StdResult, SubMsg, SubMsgResult, WasmMsg,
};
use cw_multi_test::{App, Contract, Executor};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sylvia::contract;

pub const VOTE_CODE_ID: Item<u64> = Item::new("vote_code_id");
pub const VOTES_COUNT: Item<u32> = Item::new("votes_count");

pub const VOTE_INSTANTIATE_ID: u64 = 1;

pub struct AdminContract {}

pub struct VoteContract {}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct VotesCountResp {
    pub votes_count: u32,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProposeAdminResp {
    pub vote_addr: Addr,
}

#[contract(module=admin, error=StdError)]
impl AdminContract {
    #[msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        vote_code_id: u64,
    ) -> StdResult<Response> {
        let (deps, ..) = ctx;
        VOTE_CODE_ID.save(deps.storage, &vote_code_id)?;
        VOTES_COUNT.save(deps.storage, &0)?;
        Ok(Response::new())
    }

    #[msg(exec)]
    pub fn create_vote(&self, ctx: (DepsMut, Env, MessageInfo)) -> StdResult<Response> {
        let (deps, _env, info) = ctx;
        let msg = vote::InstantiateMsg {};

        let msg = WasmMsg::Instantiate {
            admin: None,
            code_id: VOTE_CODE_ID.load(deps.storage)?,
            msg: to_binary(&msg)?,
            funds: vec![],
            label: format!("admin-{}", info.sender),
        };

        let resp = Response::new()
            .add_submessage(SubMsg::reply_on_success(msg, VOTE_INSTANTIATE_ID))
            .add_attribute("action", "create_vote")
            .add_attribute("sender", info.sender);

        Ok(resp)
    }

    #[msg(query)]
    pub fn votes_count(&self, ctx: (Deps, Env)) -> StdResult<VotesCountResp> {
        let (deps, _) = ctx;
        let votes_count = VOTES_COUNT.load(deps.storage)?;
        let resp = VotesCountResp { votes_count };
        Ok(resp)
    }

    #[msg(reply)]
    pub fn reply(&self, ctx: (DepsMut, Env), _msg: SubMsgResult) -> StdResult<Response> {
        let (deps, _) = ctx;

        VOTES_COUNT.update(deps.storage, |mut c| -> StdResult<_> {
            c += 1;
            Ok(c)
        })?;

        Ok(Response::new())
    }
}

#[contract(module=vote, error=StdError)]
impl VoteContract {
    #[msg(instantiate)]
    pub fn instantiate(&self, _ctx: (DepsMut, Env, MessageInfo)) -> StdResult<Response> {
        Ok(Response::new())
    }
}

impl Contract<Empty> for AdminContract {
    fn execute(
        &self,
        deps: cosmwasm_std::DepsMut<Empty>,
        env: cosmwasm_std::Env,
        info: cosmwasm_std::MessageInfo,
        msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        from_slice::<self::admin::ContractExecMsg>(&msg)?
            .dispatch(self, (deps, env, info))
            .map_err(Into::into)
    }

    fn instantiate(
        &self,
        deps: cosmwasm_std::DepsMut<Empty>,
        env: cosmwasm_std::Env,
        info: cosmwasm_std::MessageInfo,
        msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        from_slice::<self::admin::InstantiateMsg>(&msg)?
            .dispatch(self, (deps, env, info))
            .map_err(Into::into)
    }

    fn query(
        &self,
        deps: cosmwasm_std::Deps<Empty>,
        env: cosmwasm_std::Env,
        msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Binary> {
        from_slice::<self::admin::ContractQueryMsg>(&msg)?
            .dispatch(self, (deps, env))
            .map_err(Into::into)
    }

    fn sudo(
        &self,
        _deps: cosmwasm_std::DepsMut<Empty>,
        _env: cosmwasm_std::Env,
        _msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        bail!("sudo not implemented for contract")
    }

    fn reply(
        &self,
        deps: cosmwasm_std::DepsMut<Empty>,
        env: cosmwasm_std::Env,
        msg: cosmwasm_std::Reply,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        match msg.id {
            VOTE_INSTANTIATE_ID => self.reply((deps, env), msg.result).map_err(Into::into),
            _ => Err(anyhow::Error::new(StdError::generic_err(
                "unknown reply id",
            ))),
        }
    }

    fn migrate(
        &self,
        _deps: cosmwasm_std::DepsMut<Empty>,
        _env: cosmwasm_std::Env,
        _msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        bail!("migrate not implemented for contract")
    }
}

impl Contract<Empty> for VoteContract {
    fn execute(
        &self,
        deps: cosmwasm_std::DepsMut<Empty>,
        env: cosmwasm_std::Env,
        info: cosmwasm_std::MessageInfo,
        msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        from_slice::<self::vote::ContractExecMsg>(&msg)?
            .dispatch(self, (deps, env, info))
            .map_err(Into::into)
    }

    fn instantiate(
        &self,
        deps: cosmwasm_std::DepsMut<Empty>,
        env: cosmwasm_std::Env,
        info: cosmwasm_std::MessageInfo,
        msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        from_slice::<self::vote::InstantiateMsg>(&msg)?
            .dispatch(self, (deps, env, info))
            .map_err(Into::into)
    }

    fn query(
        &self,
        deps: cosmwasm_std::Deps<Empty>,
        env: cosmwasm_std::Env,
        msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Binary> {
        from_slice::<self::vote::ContractQueryMsg>(&msg)?
            .dispatch(self, (deps, env))
            .map_err(Into::into)
    }

    fn sudo(
        &self,
        _deps: cosmwasm_std::DepsMut<Empty>,
        _env: cosmwasm_std::Env,
        _msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        bail!("sudo not implemented for contract")
    }

    fn reply(
        &self,
        _deps: cosmwasm_std::DepsMut<Empty>,
        _env: cosmwasm_std::Env,
        _msg: cosmwasm_std::Reply,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        bail!("reply not implemented for contract")
    }

    fn migrate(
        &self,
        _deps: cosmwasm_std::DepsMut<Empty>,
        _env: cosmwasm_std::Env,
        _msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        bail!("migrate not implemented for contract")
    }
}

#[test]
fn update_count_in_reply() {
    let mut app = App::default();
    let admin_code_id = app.store_code(Box::new(AdminContract {}));
    let vote_code_id = app.store_code(Box::new(VoteContract {}));

    let admin = app
        .instantiate_contract(
            admin_code_id,
            Addr::unchecked("owner"),
            &admin::InstantiateMsg { vote_code_id },
            &[],
            "admin",
            None,
        )
        .unwrap();

    let resp: VotesCountResp = app
        .wrap()
        .query_wasm_smart(admin.clone(), &admin::QueryMsg::VotesCount {})
        .unwrap();

    assert_eq!(resp.votes_count, 0);

    app.execute_contract(
        Addr::unchecked("owner"),
        admin.clone(),
        &admin::ExecMsg::CreateVote {},
        &[],
    )
    .unwrap();

    let resp: VotesCountResp = app
        .wrap()
        .query_wasm_smart(admin, &admin::QueryMsg::VotesCount {})
        .unwrap();

    assert_eq!(resp.votes_count, 1);
}

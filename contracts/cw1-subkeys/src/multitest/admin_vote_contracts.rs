use anyhow::{bail, Result as AnyResult};
use cosmwasm_std::{
    from_slice, to_binary, Addr, Decimal, Deps, DepsMut, Empty, Env, MessageInfo, Response,
    StdError, StdResult, SubMsg, SubMsgResult, Timestamp, WasmMsg,
};
use cw_multi_test::Contract;
use cw_storage_plus::{Item, Map};
use cw_utils::parse_instantiate_response_data;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sylvia::contract;

pub const ADMINS: Map<Addr, Timestamp> = Map::new("admins");
pub const VOTE_CODE_ID: Item<u64> = Item::new("vote_code_id");
pub const QUORUM: Item<Decimal> = Item::new("quorum");
pub const PROPOSED_ADMIN: Item<Addr> = Item::new("proposed_admin");
pub const VOTE_OWNER: Item<Addr> = Item::new("vote_owner");
pub const START_TIME: Item<Timestamp> = Item::new("start_time");
pub const REQUIRED_VOTES: Item<Decimal> = Item::new("required_approvals");
pub const PENDING_VOTES: Map<Addr, Addr> = Map::new("pending_votes");

pub const VOTE_INSTANTIATE_ID: u64 = 1;

pub struct AdminContract {}

pub struct VoteContract {}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AdminsListResp {
    pub admins: Vec<Addr>,
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
        admins: Vec<String>,
        vote_code_id: u64,
        quorum: Decimal,
    ) -> StdResult<Response> {
        let (deps, env, _) = ctx;
        for addr in admins.into_iter() {
            ADMINS.save(
                deps.storage,
                deps.api.addr_validate(&addr)?,
                &env.block.time,
            )?;
        }
        VOTE_CODE_ID.save(deps.storage, &vote_code_id)?;
        QUORUM.save(deps.storage, &quorum)?;
        Ok(Response::new())
    }

    #[msg(exec)]
    pub fn propose_admin(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        admin_code_id: u64,
        addr: String,
    ) -> StdResult<Response> {
        let (deps, _env, info) = ctx;
        let msg = vote::InstantiateMsg {
            quorum: QUORUM.load(deps.storage)?,
            proposed_admin: addr,
            admin_code_id,
        };

        let msg = WasmMsg::Instantiate {
            admin: None,
            code_id: VOTE_CODE_ID.load(deps.storage)?,
            msg: to_binary(&msg)?,
            funds: vec![],
            label: format!("admin-{}", info.sender),
        };

        let resp = Response::new()
            .add_submessage(SubMsg::reply_on_success(msg, VOTE_INSTANTIATE_ID))
            .add_attribute("action", "propose_admin")
            .add_attribute("sender", info.sender);

        Ok(resp)
    }

    #[msg(query)]
    pub fn admins_list(&self, ctx: (Deps, Env)) -> StdResult<AdminsListResp> {
        let (deps, _) = ctx;
        let admins: Vec<Addr> = ADMINS
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .filter_map(|admin| admin.ok())
            .map(|(addr, _)| addr)
            .collect();
        let resp = AdminsListResp { admins };
        Ok(resp)
    }

    #[msg(reply)]
    pub fn vote_instantiated(&self, ctx: (DepsMut, Env), msg: SubMsgResult) -> StdResult<Response> {
        let (deps, _env) = ctx;
        let resp = match msg.into_result() {
            Ok(resp) => resp,
            Err(err) => return Err(StdError::generic_err(err)),
        };

        let data = resp
            .data
            .ok_or_else(|| StdError::generic_err("No instantiate response data"))?;

        let resp = parse_instantiate_response_data(&data)
            .map_err(|err| StdError::generic_err(err.to_string()))?;
        let vote_addr = Addr::unchecked(&resp.contract_address);

        let proposed_admin = PROPOSED_ADMIN.query(&deps.querier, vote_addr.clone())?;
        PENDING_VOTES.save(deps.storage, vote_addr.clone(), &proposed_admin)?;

        let resp = Response::new().set_data(to_binary(&ProposeAdminResp { vote_addr })?);
        Ok(resp)
    }
}

#[contract(module=vote, error=StdError)]
impl VoteContract {
    #[msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        quorum: Decimal,
        proposed_admin: String,
        #[allow(unused_variables)] admin_code_id: u64,
    ) -> StdResult<Response> {
        let (deps, env, info) = ctx;
        PROPOSED_ADMIN.save(deps.storage, &deps.api.addr_validate(&proposed_admin)?)?;
        START_TIME.save(deps.storage, &env.block.time)?;
        VOTE_OWNER.save(deps.storage, &info.sender)?;

        let vote_owner = &info.sender;

        let resp: AdminsListResp = deps
            .querier
            .query_wasm_smart(vote_owner, &admin::QueryMsg::AdminsList {})?;

        let admins_decimals = match Decimal::from_atomics(resp.admins.len() as u128, 0) {
            Ok(val) => val,
            Err(err) => return Err(StdError::generic_err(err.to_string())),
        };

        let required_votes = quorum * admins_decimals;

        REQUIRED_VOTES.save(deps.storage, &required_votes)?;
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
            VOTE_INSTANTIATE_ID => self
                .vote_instantiated((deps, env), msg.result)
                .map_err(Into::into),
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

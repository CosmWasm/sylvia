use cosmwasm_std::{Response, StdError, StdResult};
use sylvia::contract;
use sylvia::types::{ExecCtx, InstantiateCtx, QueryCtx, SudoCtx};

mod interface1 {
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::interface;
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};

    #[interface]
    #[sv::custom(msg=cosmwasm_std::Empty, query=cosmwasm_std::Empty)]
    pub trait SylviaInterface1 {
        type Error: From<StdError>;

        #[sv::msg(exec)]
        fn interface1_method_exec(&self, _ctx: ExecCtx) -> StdResult<Response>;

        #[sv::msg(query)]
        fn interface1_method_query(&self, _ctx: QueryCtx) -> StdResult<Response>;

        #[sv::msg(sudo)]
        fn interface1_method_sudo(&self, _ctx: SudoCtx) -> StdResult<Response>;
    }
}

mod interface2 {
    use cosmwasm_std::{Response, StdError, StdResult};
    use sylvia::interface;
    use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};

    #[interface]
    #[sv::custom(msg=cosmwasm_std::Empty, query=cosmwasm_std::Empty)]
    pub trait SylviaInterface2 {
        type Error: From<StdError>;

        #[sv::msg(exec)]
        fn interface2_method_exec(&self, _ctx: ExecCtx) -> StdResult<Response>;

        #[sv::msg(query)]
        fn interface2_method_query(&self, _ctx: QueryCtx) -> StdResult<Response>;

        #[sv::msg(sudo)]
        fn interface2_method_sudo(&self, _ctx: SudoCtx) -> StdResult<Response>;
    }
}

pub struct Contract;

#[contract]
#[sv::messages(interface1)]
#[sv::messages(interface2)]
impl Contract {
    pub fn new() -> Self {
        Self
    }

    #[sv::msg(instantiate)]
    fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[sv::msg(exec)]
    fn contract_method_exec(&self, _ctx: ExecCtx) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[sv::msg(query)]
    fn contract_method_query(&self, _ctx: QueryCtx) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[sv::msg(sudo)]
    fn contract_method_sudo(&self, _ctx: SudoCtx) -> StdResult<Response> {
        Ok(Response::new())
    }
}

impl interface1::SylviaInterface1 for Contract {
    type Error = StdError;

    fn interface1_method_exec(&self, _ctx: ExecCtx) -> StdResult<Response> {
        Ok(Response::new())
    }

    fn interface1_method_query(&self, _ctx: QueryCtx) -> StdResult<Response> {
        Ok(Response::new())
    }

    fn interface1_method_sudo(&self, _ctx: SudoCtx) -> StdResult<Response> {
        Ok(Response::new())
    }
}

impl interface2::SylviaInterface2 for Contract {
    type Error = StdError;

    fn interface2_method_exec(&self, _ctx: ExecCtx) -> StdResult<Response> {
        Ok(Response::new())
    }

    fn interface2_method_query(&self, _ctx: QueryCtx) -> StdResult<Response> {
        Ok(Response::new())
    }

    fn interface2_method_sudo(&self, _ctx: SudoCtx) -> StdResult<Response> {
        Ok(Response::new())
    }
}

impl Default for Contract {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn check_from_trait_implementations() {
    let _ =
        sv::ContractExecMsg::from(interface1::sv::SylviaInterface1ExecMsg::Interface1MethodExec {});
    let _ = sv::ContractQueryMsg::from(
        interface1::sv::SylviaInterface1QueryMsg::Interface1MethodQuery {},
    );
    let _ =
        sv::ContractSudoMsg::from(interface1::sv::SylviaInterface1SudoMsg::Interface1MethodSudo {});

    let _ =
        sv::ContractExecMsg::from(interface2::sv::SylviaInterface2ExecMsg::Interface2MethodExec {});
    let _ = sv::ContractQueryMsg::from(
        interface2::sv::SylviaInterface2QueryMsg::Interface2MethodQuery {},
    );
    let _ =
        sv::ContractSudoMsg::from(interface2::sv::SylviaInterface2SudoMsg::Interface2MethodSudo {});

    let _ = sv::ContractExecMsg::from(sv::ExecMsg::ContractMethodExec {});
    let _ = sv::ContractQueryMsg::from(sv::QueryMsg::ContractMethodQuery {});
    let _ = sv::ContractSudoMsg::from(sv::SudoMsg::ContractMethodSudo {});
}

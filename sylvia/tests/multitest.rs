#![cfg(feature = "mt")]

use cosmwasm_std::{CodeInfoResponse, Empty, Response, StdResult};
use std::marker::PhantomData;
use sylvia::multitest::App;
use sylvia::types::InstantiateCtx;
use sylvia_derive::contract;

pub struct SomeContract<ParamT> {
    _phantom: PhantomData<ParamT>,
}

#[contract]
impl<ParamT> SomeContract<ParamT>
where
    ParamT: sylvia::types::CustomMsg + 'static,
{
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx, _param: ParamT) -> StdResult<Response> {
        Ok(Response::new())
    }
}

#[test]
fn instantiate_with_salt() {
    let owner = "owner";
    let salt = "sylvia OP".as_bytes();

    let app = App::default();

    let code_id = sv::mt::CodeId::<Empty, _>::store_code(&app);

    let _: sylvia::multitest::Proxy<_, SomeContract<Empty>> = code_id
        .instantiate(Empty {})
        .with_salt(salt)
        .call(owner)
        .unwrap();
}

#[test]
fn code_info() {
    let app = App::default();

    let code_id = sv::mt::CodeId::<Empty, _>::store_code(&app);

    let _: CodeInfoResponse = code_id.code_info().unwrap();
    let _: CodeInfoResponse = app.code_info(code_id.code_id()).unwrap();
}

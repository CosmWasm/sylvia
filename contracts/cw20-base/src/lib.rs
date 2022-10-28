mod responses;

use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};
use responses::{BalanceResponse, TokenInfoResponse};
use sylvia::{interface, schemars};

#[interface]
pub trait Cw20Base {
    type Error: From<StdError>;

    /// Transfer is a base message to move tokens to another account without triggering actions
    #[msg(exec)]
    fn transfer(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        recipient: String,
        amount: Uint128,
    ) -> Result<Response, Self::Error>;

    /// Burn is a base message to destroy tokens forever
    #[msg(exec)]
    fn burn(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        new_minteramount: Uint128,
    ) -> Result<Response, Self::Error>;

    /// Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    #[msg(exec)]
    fn send(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        contract: String,
        amount: Uint128,
        msg: Binary,
    ) -> Result<Response, Self::Error>;

    /// Returns the current balance of the given address, 0 if unset.
    #[msg(query)]
    fn balance(&self, ctx: (Deps, Env), address: String) -> StdResult<BalanceResponse>;

    /// Returns metadata on the contract - name, decimals, supply, etc.
    #[msg(query)]
    fn token_info(&self, ctx: (Deps, Env)) -> StdResult<TokenInfoResponse>;
}

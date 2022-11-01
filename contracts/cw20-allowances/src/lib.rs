use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};
use cw20::{AllAllowancesResponse, AllSpenderAllowancesResponse, AllowanceResponse};
use cw_utils::Expiration;
use sylvia::{interface, schemars};

#[interface]
pub trait Cw20Allowances {
    type Error: From<StdError>;

    /// Allows spender to access an additional amount tokens from the owner's (env.sender) account.
    /// If expires is Some(), overwrites current allowance expiration with this one.
    #[msg(exec)]
    fn increase_allowance(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    ) -> Result<Response, Self::Error>;

    /// Lowers the spender's access of tokens from the owner's (env.sender) account by amount.
    /// If expires is Some(), overwrites current allowance expiration with this one.
    #[msg(exec)]
    fn decrease_allowance(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    ) -> Result<Response, Self::Error>;

    /// Transfers amount tokens from owner -> recipient
    /// if `env.sender` has sufficient pre-approval.
    #[msg(exec)]
    fn transfer_from(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        owner: String,
        recipient: String,
        amount: Uint128,
    ) -> Result<Response, Self::Error>;

    /// Sends amount tokens from owner -> contract
    /// if `env.sender` has sufficient pre-approval.
    #[msg(exec)]
    fn send_from(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        owner: String,
        contract: String,
        amount: Uint128,
        msg: Binary,
    ) -> Result<Response, Self::Error>;

    /// Destroys amount of tokens forever
    #[msg(exec)]
    fn burn_from(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        owner: String,
        amount: Uint128,
    ) -> Result<Response, Self::Error>;

    /// Returns how much spender can use from owner account, 0 if unset.
    #[msg(query)]
    fn allowance(
        &self,
        ctx: (Deps, Env),
        owner: String,
        spender: String,
    ) -> StdResult<AllowanceResponse>;

    /// Returns all allowances this owner has approved. Supports pagination.
    #[msg(query)]
    fn all_allowances(
        &self,
        ctx: (Deps, Env),
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<AllAllowancesResponse>;

    /// Returns all allowances this spender has been granted. Supports pagination.
    #[msg(query)]
    fn all_spender_allowances(
        &self,
        ctx: (Deps, Env),
        spender: String,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<AllSpenderAllowancesResponse>;
}

pub mod responses;

use cosmwasm_std::{Binary, Response, StdError, StdResult, Uint128};
use cw_utils::Expiration;
use responses::{
    AllAccountsResponse, AllAllowancesResponse, AllSpenderAllowancesResponse, AllowanceResponse,
};
use sylvia::types::{CustomMsg, CustomQuery, ExecCtx, QueryCtx};
use sylvia::{interface, schemars};

#[interface]
pub trait Cw20Allowances {
    type Error: From<StdError>;
    type ExecC: CustomMsg;
    type QueryC: CustomQuery;

    /// Allows spender to access an additional amount tokens from the owner's (env.sender) account.
    /// If expires is Some(), overwrites current allowance expiration with this one.
    #[sv::msg(exec)]
    fn increase_allowance(
        &self,
        ctx: ExecCtx<Self::QueryC>,
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    ) -> Result<Response<Self::ExecC>, Self::Error>;

    /// Lowers the spender's access of tokens from the owner's (env.sender) account by amount.
    /// If expires is Some(), overwrites current allowance expiration with this one.
    #[sv::msg(exec)]
    fn decrease_allowance(
        &self,
        ctx: ExecCtx<Self::QueryC>,
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    ) -> Result<Response<Self::ExecC>, Self::Error>;

    /// Transfers amount tokens from owner -> recipient
    /// if `env.sender` has sufficient pre-approval.
    #[sv::msg(exec)]
    fn transfer_from(
        &self,
        ctx: ExecCtx<Self::QueryC>,
        owner: String,
        recipient: String,
        amount: Uint128,
    ) -> Result<Response<Self::ExecC>, Self::Error>;

    /// Sends amount tokens from owner -> contract
    /// if `env.sender` has sufficient pre-approval.
    #[sv::msg(exec)]
    fn send_from(
        &self,
        ctx: ExecCtx<Self::QueryC>,
        owner: String,
        contract: String,
        amount: Uint128,
        msg: Binary,
    ) -> Result<Response<Self::ExecC>, Self::Error>;

    /// Destroys amount of tokens forever
    #[sv::msg(exec)]
    fn burn_from(
        &self,
        ctx: ExecCtx<Self::QueryC>,
        owner: String,
        amount: Uint128,
    ) -> Result<Response<Self::ExecC>, Self::Error>;

    /// Returns how much spender can use from owner account, 0 if unset.
    #[sv::msg(query)]
    fn allowance(
        &self,
        ctx: QueryCtx<Self::QueryC>,
        owner: String,
        spender: String,
    ) -> StdResult<AllowanceResponse>;

    /// Returns all allowances this owner has approved. Supports pagination.
    #[sv::msg(query)]
    fn all_allowances(
        &self,
        ctx: QueryCtx<Self::QueryC>,
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<AllAllowancesResponse>;

    /// Returns all allowances this spender has been granted. Supports pagination.
    #[sv::msg(query)]
    fn all_spender_allowances(
        &self,
        ctx: QueryCtx<Self::QueryC>,
        spender: String,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<AllSpenderAllowancesResponse>;

    /// Returns all accounts that have balances. Supports pagination.
    #[sv::msg(query)]
    fn all_accounts(
        &self,
        ctx: QueryCtx<Self::QueryC>,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<AllAccountsResponse>;
}

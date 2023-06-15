pub mod responses;

use cosmwasm_std::{CustomMsg, Response, StdError, StdResult, Uint128};
use responses::MinterResponse;
use sylvia::types::{ExecCtx, QueryCtx};
use sylvia::{interface, schemars};

#[interface]
pub trait Cw20Minting {
    type Error: From<StdError>;
    type ExecC: CustomMsg;

    /// If authorized, creates amount new tokens and adds to the recipient balance.
    #[msg(exec)]
    fn mint(
        &self,
        ctx: ExecCtx,
        recipient: String,
        amount: Uint128,
    ) -> Result<Response<Self::ExecC>, Self::Error>;

    /// The current minter may set a new minter.
    /// Setting the minter to None will remove the token's minter forever.
    #[msg(exec)]
    fn update_minter(
        &self,
        ctx: ExecCtx,
        new_minter: Option<String>,
    ) -> Result<Response<Self::ExecC>, Self::Error>;

    /// Returns who can mint and the hard cap on maximum tokens after minting.
    #[msg(query)]
    fn minter(&self, ctx: QueryCtx) -> StdResult<Option<MinterResponse>>;
}

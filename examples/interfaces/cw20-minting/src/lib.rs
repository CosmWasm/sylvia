pub mod responses;

use responses::MinterResponse;
use sylvia::cw_std::{Response, StdError, StdResult, Uint128};
use sylvia::interface;
use sylvia::types::{CustomMsg, CustomQuery, ExecCtx, QueryCtx};

#[interface]
pub trait Cw20Minting {
    type Error: From<StdError>;
    type ExecC: CustomMsg;
    type QueryC: CustomQuery;

    /// If authorized, creates amount new tokens and adds to the recipient balance.
    #[sv::msg(exec)]
    fn mint(
        &self,
        ctx: ExecCtx<Self::QueryC>,
        recipient: String,
        amount: Uint128,
    ) -> Result<Response<Self::ExecC>, Self::Error>;

    /// The current minter may set a new minter.
    /// Setting the minter to None will remove the token's minter forever.
    #[sv::msg(exec)]
    fn update_minter(
        &self,
        ctx: ExecCtx<Self::QueryC>,
        new_minter: Option<String>,
    ) -> Result<Response<Self::ExecC>, Self::Error>;

    /// Returns who can mint and the hard cap on maximum tokens after minting.
    #[sv::msg(query)]
    fn minter(&self, ctx: QueryCtx<Self::QueryC>) -> StdResult<Option<MinterResponse>>;
}

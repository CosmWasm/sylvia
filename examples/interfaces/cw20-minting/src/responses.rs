use sylvia::cw_schema::cw_serde;
use sylvia::cw_std::Uint128;

#[cw_serde(crate = "sylvia::cw_schema")]
pub struct MinterResponse {
    pub minter: String,
    /// cap is a hard cap on total supply that can be achieved by minting.
    /// Note that this refers to total_supply.
    /// If None, there is unlimited cap.
    pub cap: Option<Uint128>,
}

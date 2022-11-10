use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct MinterResponse {
    pub minter: String,
    /// cap is a hard cap on total supply that can be achieved by minting.
    /// Note that this refers to total_supply.
    /// If None, there is unlimited cap.
    pub cap: Option<u128>,
}

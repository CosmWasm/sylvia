use cw_utils::Expiration;
use sylvia::cw_schema::cw_serde;
use sylvia::cw_std::Uint128;
use sylvia::schemars::JsonSchema;
use sylvia::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[schemars(crate = "sylvia::cw_schema::schemars")]
#[serde(crate = "sylvia::serde")]
pub struct AllowanceResponse {
    pub allowance: Uint128,
    pub expires: Expiration,
}

#[cw_serde(crate = "sylvia::cw_schema")]
pub struct AllowanceInfo {
    pub spender: String,
    pub allowance: Uint128,
    pub expires: Expiration,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[schemars(crate = "sylvia::cw_schema::schemars")]
#[serde(crate = "sylvia::serde")]
pub struct AllAllowancesResponse {
    pub allowances: Vec<AllowanceInfo>,
}

#[cw_serde(crate = "sylvia::cw_schema")]
pub struct SpenderAllowanceInfo {
    pub owner: String,
    pub allowance: Uint128,
    pub expires: Expiration,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[schemars(crate = "sylvia::cw_schema::schemars")]
#[serde(crate = "sylvia::serde")]
pub struct AllSpenderAllowancesResponse {
    pub allowances: Vec<SpenderAllowanceInfo>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug, Default)]
#[schemars(crate = "sylvia::cw_schema::schemars")]
#[serde(crate = "sylvia::serde")]
pub struct AllAccountsResponse {
    pub accounts: Vec<String>,
}

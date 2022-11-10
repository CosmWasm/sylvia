use cosmwasm_schema::cw_serde;
use cw_utils::Expiration;
use serde::{Deserialize, Serialize};
use sylvia::schemars;

#[cw_serde]
pub struct AllowanceInfo {
    pub spender: String,
    pub allowance: u128,
    pub expires: Expiration,
}

#[cw_serde]
pub struct SpenderAllowanceInfo {
    pub owner: String,
    pub allowance: u128,
    pub expires: Expiration,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, schemars::JsonSchema, Debug, Default)]
pub struct AllowanceResponse {
    pub allowance: u128,
    pub expires: Expiration,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, schemars::JsonSchema, Debug, Default)]
pub struct AllAllowancesResponse {
    pub allowances: Vec<AllowanceInfo>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, schemars::JsonSchema, Debug, Default)]
pub struct AllSpenderAllowancesResponse {
    pub allowances: Vec<SpenderAllowanceInfo>,
}

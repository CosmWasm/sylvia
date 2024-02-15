use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CustomMsg, CustomQuery};

#[cw_serde]
pub struct CountResponse {
    pub count: u64,
}

#[cw_serde]
pub enum CounterMsg {
    Increment {},
}

#[cw_serde]
pub enum CounterSudo {
    Increment {},
}

#[cw_serde]
pub enum CounterQuery {
    Sudo {},
    Exec {},
}

impl CustomMsg for CounterMsg {}
impl CustomMsg for CounterSudo {}
impl CustomQuery for CounterQuery {}

use sylvia::cw_schema::cw_serde;
use sylvia::cw_std::{CustomMsg, CustomQuery};

#[cw_serde(crate = "sylvia::cw_schema")]
pub struct CountResponse {
    pub count: u64,
}

#[cw_serde(crate = "sylvia::cw_schema")]
pub enum CounterMsg {
    Increment {},
}

#[cw_serde(crate = "sylvia::cw_schema")]
pub enum CounterQuery {
    Count {},
}

impl CustomMsg for CounterMsg {}

impl CustomQuery for CounterQuery {}

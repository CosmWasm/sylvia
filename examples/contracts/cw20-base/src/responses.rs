use std::fmt;
use sylvia::cw_schema::cw_serde;
use sylvia::cw_std::{to_json_binary, Addr, Binary, CosmosMsg, StdResult, Uint128, WasmMsg};

/// Cw20ReceiveMsg should be de/serialized under `Receive()` variant in a ExecuteMsg
#[cw_serde(crate = "sylvia::cw_schema")]
pub struct Cw20ReceiveMsg {
    pub sender: String,
    pub amount: Uint128,
    pub msg: Binary,
}

impl Cw20ReceiveMsg {
    /// serializes the message
    pub fn into_json_binary(self) -> StdResult<Binary> {
        let msg = ReceiverExecuteMsg::Receive(self);
        to_json_binary(&msg)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg<T: Into<String>, E>(self, contract_addr: T) -> StdResult<CosmosMsg<E>> {
        let msg = self.into_json_binary()?;
        let execute = WasmMsg::Execute {
            contract_addr: contract_addr.into(),
            msg,
            funds: vec![],
        };
        Ok(execute.into())
    }
}

// This is just a helper to properly serialize the above message
#[cw_serde(crate = "sylvia::cw_schema")]
enum ReceiverExecuteMsg {
    Receive(Cw20ReceiveMsg),
}

#[cw_serde(crate = "sylvia::cw_schema")]
pub struct Cw20Coin {
    pub address: String,
    pub amount: Uint128,
}

impl Cw20Coin {
    pub fn is_empty(&self) -> bool {
        self.amount == Uint128::zero()
    }
}

impl fmt::Display for Cw20Coin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "address: {}, amount: {}", self.address, self.amount)
    }
}

#[cw_serde(crate = "sylvia::cw_schema")]
pub struct Cw20CoinVerified {
    pub address: Addr,
    pub amount: Uint128,
}

impl Cw20CoinVerified {
    pub fn is_empty(&self) -> bool {
        self.amount == Uint128::zero()
    }
}

impl fmt::Display for Cw20CoinVerified {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "address: {}, amount: {}", self.address, self.amount)
    }
}

#[cw_serde(crate = "sylvia::cw_schema")]
pub struct BalanceResponse {
    pub balance: Uint128,
}

#[cw_serde(crate = "sylvia::cw_schema")]
pub struct TokenInfoResponse {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
}

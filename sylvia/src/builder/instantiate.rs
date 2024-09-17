use cosmwasm_std::{Binary, Coin, WasmMsg};

/// Builder for the CW instantiate message.
pub struct InstantiateBuilder {
    /// Serialized instantiate message.
    msg: Binary,
    /// Id of the contract stored on the chain.
    code_id: u64,
    /// Admin address of the contract.
    admin: Option<String>,
    /// A human-readable label for the contract.
    label: Option<String>,
    /// Funds sent to the contract.
    funds: Vec<Coin>,
}

impl InstantiateBuilder {
    /// Create a new [InstantiateBuilder] with the given message and code id.
    /// The message should be a serialized instance of the `InstantiateMsg` struct of the contract.
    pub fn new(msg: Binary, code_id: u64) -> Self {
        Self {
            msg,
            code_id,
            admin: None,
            label: None,
            funds: vec![],
        }
    }

    /// Set label for the contract.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set admin address for the contract.
    pub fn with_admin(mut self, admin: String) -> Self {
        self.admin = Some(admin);
        self
    }

    /// Set funds to be sent to the contract.
    pub fn with_funds(mut self, funds: Vec<Coin>) -> Self {
        self.funds = funds;
        self
    }

    /// Build [WasmMsg::Instantiate] message.
    pub fn build(self) -> WasmMsg {
        WasmMsg::Instantiate {
            code_id: self.code_id,
            msg: self.msg,
            admin: self.admin,
            label: self.label.unwrap_or_default(),
            funds: self.funds,
        }
    }

    #[cfg(feature = "cosmwasm_1_2")]
    /// Build [WasmMsg::Instantiate2] message.
    pub fn build2(self, salt: Binary) -> WasmMsg {
        WasmMsg::Instantiate2 {
            code_id: self.code_id,
            msg: self.msg,
            admin: self.admin,
            label: self.label.unwrap_or_default(),
            funds: self.funds,
            salt,
        }
    }
}

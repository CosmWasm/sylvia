//! Framework for creating CosmWasm Smart Contract with high-level abstraction layer
//!
//! Most of implementation lies in `cw-derive-ng` crate which is reexported here

pub use cosmwasm_std as cw_std;
pub use schemars;
pub use serde;
pub use serde_cw_value as serde_value;
pub use sylvia_derive::{contract, interface};

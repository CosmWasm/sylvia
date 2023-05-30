//! Framework for creating CosmWasm Smart Contract with high-level abstraction layer
//!
//! Most of implementation lies in `cw-derive-ng` crate which is reexported here

pub mod into_deps;
#[cfg(feature = "mt")]
pub mod multitest;
pub mod types;
pub mod utils;

#[cfg(feature = "mt")]
pub use anyhow;
pub use cosmwasm_std as cw_std;
#[cfg(feature = "mt")]
pub use cw_multi_test;
#[cfg(feature = "mt")]
pub use derivative;
pub use schemars;
pub use serde;
pub use serde_cw_value as serde_value;
pub use serde_json_wasm as serde_json;
pub use sylvia_derive::{contract, entry_points, interface};

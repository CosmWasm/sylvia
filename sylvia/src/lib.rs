#![cfg_attr(docsrs, feature(doc_cfg))]
//! Framework for creating CosmWasm Smart Contract with high-level abstraction layer.
//!
//! Most of implementation lies in `sylvia-derive` crate which is reexported here.

pub mod builder;
pub mod ctx;
pub mod into_response;
#[cfg_attr(docsrs, doc(cfg(feature = "mt")))]
#[cfg(feature = "mt")]
pub mod multitest;
pub mod types;
pub mod utils;

#[cfg_attr(docsrs, doc(cfg(feature = "mt")))]
#[cfg(feature = "mt")]
pub use anyhow;
#[cfg_attr(docsrs, doc(cfg(feature = "mt")))]
#[cfg(feature = "mt")]
pub use cw_multi_test;
pub use cw_utils;
pub use sylvia_derive::{contract, entry_points, interface};
pub use {
    cosmwasm_schema as cw_schema, cosmwasm_std as cw_std, schemars, serde,
    serde_cw_value as serde_value,
};

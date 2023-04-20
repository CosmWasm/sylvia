pub mod allowances;
pub mod contract;
pub mod error;
pub mod marketing;
pub mod minting;
pub mod responses;
pub mod validation;

#[cfg(any(test, feature = "tests"))]
mod multitest;

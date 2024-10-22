#![cfg_attr(docsrs, feature(doc_cfg))]
//! Framework for creating CosmWasm Smart Contract with high-level abstraction layer
//!
//! Most of implementation lies in `sylvia-derive` crate which is reexported here

pub mod builder;
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
#[cfg_attr(docsrs, doc(cfg(feature = "cosmwasm_1_2")))]
#[cfg(feature = "cosmwasm_1_2")]
pub use cw_utils;
#[cfg_attr(docsrs, doc(cfg(feature = "mt")))]
#[cfg(feature = "mt")]
pub use derivative;
pub use sylvia_derive::{contract, entry_points, interface};
pub use {
    cosmwasm_schema as cw_schema, cosmwasm_std as cw_std, schemars, serde,
    serde_cw_value as serde_value, serde_json_wasm as serde_json,
};

pub mod replies {
    use cosmwasm_std::{DepsMut, Empty, Env, Event, MsgResponse};

    /// Represantation of `reply` context received in entry point.
    #[deprecated(
        since = "1.3.0",
        note = "This type is added temporarily to not break existing API. Since `2.0.0` it will replace the `sylvia::types::ReplyCtx` type."
    )]
    #[non_exhaustive]
    pub struct ReplyCtx<'a, C: cosmwasm_std::CustomQuery = Empty> {
        pub deps: DepsMut<'a, C>,
        pub env: Env,
        pub gas_used: u64,
        pub events: Vec<Event>,
        pub msg_responses: Vec<MsgResponse>,
    }

    #[allow(deprecated)]
    impl<'a, C: cosmwasm_std::CustomQuery>
        From<(DepsMut<'a, C>, Env, u64, Vec<Event>, Vec<MsgResponse>)> for ReplyCtx<'a, C>
    {
        fn from(
            (deps, env, gas_used, events, msg_responses): (
                DepsMut<'a, C>,
                Env,
                u64,
                Vec<Event>,
                Vec<MsgResponse>,
            ),
        ) -> Self {
            Self {
                deps,
                env,
                gas_used,
                events,
                msg_responses,
            }
        }
    }
}

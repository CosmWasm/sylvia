use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary};
use serde::{Deserialize, Serialize};
use sylvia::schemars;

/// This is used to display logo info, provide a link or inform there is one
/// that can be downloaded from the blockchain itself
#[cw_serde]
pub enum LogoInfo {
    /// A reference to an externally hosted logo. Must be a valid HTTP or HTTPS URL.
    Url(String),
    /// There is an embedded logo on the chain, make another call to download it.
    Embedded,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, schemars::JsonSchema, Debug, Default)]
pub struct MarketingInfoResponse {
    /// A URL pointing to the project behind this token.
    pub project: Option<String>,
    /// A longer description of the token and it's utility. Designed for tooltips or such
    pub description: Option<String>,
    /// A link to the logo, or a comment there is an on-chain logo stored
    pub logo: Option<LogoInfo>,
    /// The address (if any) who can update this data structure
    pub marketing: Option<Addr>,
}

/// When we download an embedded logo, we get this response type.
/// We expect a SPA to be able to accept this info and display it.
#[cw_serde]
pub struct DownloadLogoResponse {
    pub mime_type: String,
    pub data: Binary,
}

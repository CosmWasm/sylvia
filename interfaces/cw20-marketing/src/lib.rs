pub mod responses;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Response, StdError, StdResult};
use responses::{DownloadLogoResponse, MarketingInfoResponse};
use sylvia::types::{ExecCtx, QueryCtx};
use sylvia::{interface, schemars};

/// This is used for uploading logo data, or setting it in InstantiateData
#[cw_serde]
pub enum Logo {
    /// A reference to an externally hosted logo. Must be a valid HTTP or HTTPS URL.
    Url(String),
    /// Logo content stored on the blockchain. Enforce maximum size of 5KB on all variants
    Embedded(EmbeddedLogo),
}

/// This is used to store the logo on the blockchain in an accepted format.
/// Enforce maximum size of 5KB on all variants.
#[cw_serde]
pub enum EmbeddedLogo {
    /// Store the Logo as an SVG file. The content must conform to the spec
    /// at https://en.wikipedia.org/wiki/Scalable_Vector_Graphics
    /// (The contract should do some light-weight sanity-check validation)
    Svg(Binary),
    /// Store the Logo as a PNG file. This will likely only support up to 64x64 or so
    /// within the 5KB limit.
    Png(Binary),
}

#[interface]
pub trait Cw20Marketing {
    type Error: From<StdError>;

    /// If authorized, updates marketing metadata.
    /// Setting None/null for any of these will leave it unchanged.
    /// Setting Some("") will clear this field on the contract storage
    /// `project` - A URL pointing to the project behind this token.
    /// `description` - A longer description of the token and it's utility. Designed for tooltips or such
    /// `marketing` - The address (if any) who can update this data structure
    #[msg(exec)]
    fn update_marketing(
        &self,
        ctx: ExecCtx,
        project: Option<String>,
        description: Option<String>,
        marketing: Option<String>,
    ) -> Result<Response, Self::Error>;

    /// If set as the "marketing" role on the contract, upload a new URL, SVG, or PNG for the token
    #[msg(exec)]
    fn upload_logo(&self, ctx: ExecCtx, logo: Logo) -> Result<Response, Self::Error>;

    /// Returns more metadata on the contract to display in the client:
    /// - description, logo, project url, etc.
    #[msg(query)]
    fn marketing_info(&self, ctx: QueryCtx) -> StdResult<MarketingInfoResponse>;

    /// Downloads the embedded logo data (if stored on chain). Errors if no logo data is stored for this
    /// contract.
    #[msg(query)]
    fn download_logo(&self, ctx: QueryCtx) -> StdResult<DownloadLogoResponse>;
}

use crate::contract::Cw20Base;
use crate::error::ContractError;
use crate::validation::verify_logo;
use cosmwasm_std::{Empty, Response, StdError, StdResult};
use cw20_marketing::responses::{DownloadLogoResponse, LogoInfo, MarketingInfoResponse};
use cw20_marketing::{Cw20Marketing, EmbeddedLogo, Logo};
use sylvia::types::{ExecCtx, QueryCtx};

impl Cw20Marketing for Cw20Base {
    type Error = ContractError;
    type ExecC = Empty;
    type QueryC = Empty;

    fn update_marketing(
        &self,
        ctx: ExecCtx,
        project: Option<String>,
        description: Option<String>,
        marketing: Option<String>,
    ) -> Result<Response, Self::Error> {
        let mut marketing_info = self
            .marketing_info
            .may_load(ctx.deps.storage)?
            .ok_or(ContractError::Unauthorized)?;

        if marketing_info
            .marketing
            .as_ref()
            .ok_or(ContractError::Unauthorized)?
            != ctx.info.sender
        {
            return Err(ContractError::Unauthorized);
        }

        match project {
            Some(empty) if empty.trim().is_empty() => marketing_info.project = None,
            Some(project) => marketing_info.project = Some(project),
            None => (),
        }

        match description {
            Some(empty) if empty.trim().is_empty() => marketing_info.description = None,
            Some(description) => marketing_info.description = Some(description),
            None => (),
        }

        match marketing {
            Some(empty) if empty.trim().is_empty() => marketing_info.marketing = None,
            Some(marketing) => {
                marketing_info.marketing = Some(ctx.deps.api.addr_validate(&marketing)?)
            }
            None => (),
        }

        if marketing_info.project.is_none()
            && marketing_info.description.is_none()
            && marketing_info.marketing.is_none()
            && marketing_info.logo.is_none()
        {
            self.marketing_info.remove(ctx.deps.storage);
        } else {
            self.marketing_info
                .save(ctx.deps.storage, &marketing_info)?;
        }

        let res = Response::new().add_attribute("action", "update_marketing");
        Ok(res)
    }

    fn upload_logo(&self, ctx: ExecCtx, logo: Logo) -> Result<Response, Self::Error> {
        let mut marketing_info = self
            .marketing_info
            .may_load(ctx.deps.storage)?
            .ok_or(ContractError::Unauthorized)?;

        verify_logo(&logo)?;

        if marketing_info
            .marketing
            .as_ref()
            .ok_or(ContractError::Unauthorized)?
            != ctx.info.sender
        {
            return Err(ContractError::Unauthorized);
        }

        self.logo.save(ctx.deps.storage, &logo)?;

        let logo_info = match logo {
            Logo::Url(url) => LogoInfo::Url(url),
            Logo::Embedded(_) => LogoInfo::Embedded,
        };

        marketing_info.logo = Some(logo_info);
        self.marketing_info
            .save(ctx.deps.storage, &marketing_info)?;

        let res = Response::new().add_attribute("action", "upload_logo");
        Ok(res)
    }

    fn marketing_info(&self, ctx: QueryCtx) -> StdResult<MarketingInfoResponse> {
        Ok(self
            .marketing_info
            .may_load(ctx.deps.storage)?
            .unwrap_or_default())
    }

    fn download_logo(&self, ctx: QueryCtx) -> StdResult<DownloadLogoResponse> {
        let logo = self.logo.load(ctx.deps.storage)?;
        match logo {
            Logo::Embedded(EmbeddedLogo::Svg(logo)) => Ok(DownloadLogoResponse {
                mime_type: "image/svg+xml".to_owned(),
                data: logo,
            }),
            Logo::Embedded(EmbeddedLogo::Png(logo)) => Ok(DownloadLogoResponse {
                mime_type: "image/png".to_owned(),
                data: logo,
            }),
            Logo::Url(_) => Err(StdError::not_found("logo")),
        }
    }
}

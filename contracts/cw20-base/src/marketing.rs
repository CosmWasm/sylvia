use crate::contract::Cw20Base;
use crate::error::ContractError;
use crate::validation::verify_logo;
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};
use cw20_marketing::responses::{DownloadLogoResponse, LogoInfo, MarketingInfoResponse};
use cw20_marketing::{Cw20Marketing, EmbeddedLogo, Logo};
use sylvia::contract;

#[contract]
#[messages(cw20_marketing as Cw20Marketing)]
impl Cw20Marketing for Cw20Base<'_> {
    type Error = ContractError;

    #[msg(exec)]
    fn update_marketing(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        project: Option<String>,
        description: Option<String>,
        marketing: Option<String>,
    ) -> Result<Response, Self::Error> {
        let (deps, _, info) = ctx;

        let mut marketing_info = self
            .marketing_info
            .may_load(deps.storage)?
            .ok_or(ContractError::Unauthorized {})?;

        if marketing_info
            .marketing
            .as_ref()
            .ok_or(ContractError::Unauthorized {})?
            != &info.sender
        {
            return Err(ContractError::Unauthorized {});
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
            Some(marketing) => marketing_info.marketing = Some(deps.api.addr_validate(&marketing)?),
            None => (),
        }

        if marketing_info.project.is_none()
            && marketing_info.description.is_none()
            && marketing_info.marketing.is_none()
            && marketing_info.logo.is_none()
        {
            self.marketing_info.remove(deps.storage);
        } else {
            self.marketing_info.save(deps.storage, &marketing_info)?;
        }

        let res = Response::new().add_attribute("action", "update_marketing");
        Ok(res)
    }

    #[msg(exec)]
    fn upload_logo(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        logo: Logo,
    ) -> Result<Response, Self::Error> {
        let (deps, _, info) = ctx;

        let mut marketing_info = self
            .marketing_info
            .may_load(deps.storage)?
            .ok_or(ContractError::Unauthorized {})?;

        verify_logo(&logo)?;

        if marketing_info
            .marketing
            .as_ref()
            .ok_or(ContractError::Unauthorized {})?
            != &info.sender
        {
            return Err(ContractError::Unauthorized {});
        }

        self.logo.save(deps.storage, &logo)?;

        let logo_info = match logo {
            Logo::Url(url) => LogoInfo::Url(url),
            Logo::Embedded(_) => LogoInfo::Embedded,
        };

        marketing_info.logo = Some(logo_info);
        self.marketing_info.save(deps.storage, &marketing_info)?;

        let res = Response::new().add_attribute("action", "upload_logo");
        Ok(res)
    }

    #[msg(query)]
    fn marketing_info(&self, ctx: (Deps, Env)) -> StdResult<MarketingInfoResponse> {
        let (deps, _) = ctx;

        Ok(self
            .marketing_info
            .may_load(deps.storage)?
            .unwrap_or_default())
    }

    #[msg(query)]
    fn download_logo(&self, ctx: (Deps, Env)) -> StdResult<DownloadLogoResponse> {
        let (deps, _) = ctx;

        let logo = self.logo.load(deps.storage)?;
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

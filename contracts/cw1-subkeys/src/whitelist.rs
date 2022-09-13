use cw1_whitelist::whitelist::Whitelist;

use crate::contract::Cw1SubkeysContract;
use crate::error::ContractError;

impl Whitelist for Cw1SubkeysContract<'_> {
    type Error = ContractError;

    fn freeze(
        &self,
        ctx: (
            cosmwasm_std::DepsMut,
            cosmwasm_std::Env,
            cosmwasm_std::MessageInfo,
        ),
    ) -> Result<cosmwasm_std::Response, Self::Error> {
        self.whitelist.freeze(ctx).map_err(From::from)
    }

    fn update_admins(
        &self,
        ctx: (
            cosmwasm_std::DepsMut,
            cosmwasm_std::Env,
            cosmwasm_std::MessageInfo,
        ),
        admins: Vec<String>,
    ) -> Result<cosmwasm_std::Response, Self::Error> {
        self.whitelist
            .update_admins(ctx, admins)
            .map_err(From::from)
    }

    fn admin_list(
        &self,
        ctx: (cosmwasm_std::Deps, cosmwasm_std::Env),
    ) -> cosmwasm_std::StdResult<cw1_whitelist::responses::AdminListResponse> {
        self.whitelist.admin_list(ctx)
    }
}

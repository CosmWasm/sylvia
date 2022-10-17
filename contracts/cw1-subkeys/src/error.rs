use cosmwasm_std::StdError;
use cw1_whitelist::error::ContractError as WhitelistError;
use cw_utils::Expiration;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Contract is frozen")]
    ContractFrozen {},

    #[error("Cannot set to own account")]
    CannotSetOwnAccount {},

    #[error("No allowance for this account")]
    NoAllowance {},

    #[error("Allowance already expired while setting: {0}")]
    SettingExpiredAllowance(Expiration),
}

#[cfg(not(tarpaulin_include))]
impl From<WhitelistError> for ContractError {
    fn from(err: WhitelistError) -> Self {
        match err {
            WhitelistError::Unauthorized {} => ContractError::Unauthorized {},
            WhitelistError::ContractFrozen {} => ContractError::ContractFrozen {},
            WhitelistError::Std(err) => ContractError::Std(err),
        }
    }
}

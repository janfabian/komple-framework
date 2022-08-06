use cosmwasm_std::{OverflowError, StdError};
use komple_utils::{FundsError, UtilError};
use thiserror::Error;
use token_contract::ContractError as TokenContractError;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid funds")]
    InvalidFunds {},

    #[error("Invalid denom")]
    InvalidDenom {},

    #[error("Token transfer locked")]
    TransferLocked {},

    #[error("Token send locked")]
    SendLocked {},

    #[error("Token burn locked")]
    BurnLocked {},

    #[error("Token is not listed")]
    NotListed {},

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("{0}")]
    Util(#[from] UtilError),

    #[error("{0}")]
    Funds(#[from] FundsError),
}

impl From<TokenContractError> for ContractError {
    fn from(err: TokenContractError) -> ContractError {
        match err {
            TokenContractError::TransferLocked {} => ContractError::TransferLocked {},
            TokenContractError::SendLocked {} => ContractError::SendLocked {},
            TokenContractError::BurnLocked {} => ContractError::BurnLocked {},
            _ => unreachable!(),
        }
    }
}

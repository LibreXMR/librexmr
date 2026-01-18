use jsonrpc_core::types::error::Error as RpcError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("invalid compressed point")]
    InvalidPoint,
    #[error("small-order point")]
    SmallOrderPoint,
    #[error("invalid scalar encoding")]
    InvalidScalar,
}

#[derive(Debug, Error)]
pub enum XmrWalletError {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    Rpc(#[from] RpcError),
    #[error(transparent)]
    Address(#[from] monero::util::address::Error),
    #[error(transparent)]
    Crypto(#[from] CryptoError),
    #[error("unsupported operation: {0}")]
    Unsupported(&'static str),
}

impl From<monero::util::key::Error> for CryptoError {
    fn from(_value: monero::util::key::Error) -> Self {
        CryptoError::InvalidScalar
    }
}

pub type Result<T> = std::result::Result<T, XmrWalletError>;

use jsonrpc_core::types::error::Error as RpcError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("invalid compressed point")]
    InvalidPoint,
    #[error("small-order point")]
    SmallOrderPoint,
}

#[derive(Debug, Error)]
pub enum XmrWalletError {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    Rpc(#[from] RpcError),
    #[error(transparent)]
    Crypto(#[from] CryptoError),
    #[error("unsupported operation: {0}")]
    Unsupported(&'static str),
}

pub type Result<T> = std::result::Result<T, XmrWalletError>;

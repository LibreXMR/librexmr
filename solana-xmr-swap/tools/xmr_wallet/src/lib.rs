//! Monero wallet integration for real swaps.
//!
//! Auditor notes:
//! - All crypto primitives are isolated under `crypto/`.
//! - Only audited libraries are used for curve operations.
//! - RPC access is a thin wrapper over `monero-rpc`.

pub mod claim;
pub mod crypto;
pub mod rpc;
pub mod types;
pub mod watcher;

pub use crypto::{complete_adaptor, derive_spend_key, validate_point, PublicKey, SecretKey};
pub use rpc::XmrWallet;
pub use types::{CryptoError, Result, XmrWalletError};
pub use watcher::{LockWatcher, WatcherConfig, WatcherEvent, WatcherState};

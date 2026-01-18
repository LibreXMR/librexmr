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

pub use claim::{
    derive_claim_keys, execute_claim, prepare_claim_guard, validate_destination_address,
    validate_hashlock, ClaimGuard, ClaimKeys, ClaimParams,
};
pub use crypto::{
    complete_adaptor, derive_spend_key, derive_view_key, to_monero_private_key, validate_point,
    PublicKey, SecretKey,
};
pub use rpc::{RetryConfig, XmrWallet};
pub use types::{CryptoError, Result, XmrWalletError};
pub use watcher::{LockWatcher, WatcherConfig, WatcherEvent, WatcherState};

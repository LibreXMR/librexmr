//! Cryptographic primitives for Monero atomic swaps.
//!
//! Auditor notes:
//! - Uses curve25519-dalek only (Quarkslab audited).
//! - No custom scalar/point arithmetic.

mod adaptor;
mod keys;

pub use adaptor::{complete_adaptor, derive_spend_key, validate_point};
pub use keys::{PublicKey, SecretKey};

use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};
use curve25519_dalek::scalar::Scalar;

use crate::types::CryptoError;

/// Completes an adaptor signature scalar: s' = s + t.
pub fn complete_adaptor(adaptor_s: &Scalar, revealed_secret: &Scalar) -> Scalar {
    adaptor_s + revealed_secret
}

/// Derives the spend key: s_total = s_a + s_b + t.
pub fn derive_spend_key(
    alice_partial: &Scalar,
    bob_partial: &Scalar,
    revealed_secret: &Scalar,
) -> Scalar {
    alice_partial + bob_partial + revealed_secret
}

/// Validates an Ed25519 point and rejects small-order points.
pub fn validate_point(compressed: &[u8; 32]) -> Result<EdwardsPoint, CryptoError> {
    let point = CompressedEdwardsY(*compressed)
        .decompress()
        .ok_or(CryptoError::InvalidPoint)?;

    if point.is_small_order() {
        return Err(CryptoError::SmallOrderPoint);
    }

    Ok(point)
}

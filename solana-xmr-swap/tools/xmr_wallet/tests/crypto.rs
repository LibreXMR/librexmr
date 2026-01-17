use curve25519_dalek::edwards::EdwardsPoint;
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::traits::Identity;
use xmr_wallet::{complete_adaptor, derive_spend_key, validate_point};

#[test]
fn completes_adaptor_signature() {
    let adaptor = Scalar::from(7u64);
    let secret = Scalar::from(11u64);
    let completed = complete_adaptor(&adaptor, &secret);
    assert_eq!(completed, adaptor + secret);
}

#[test]
fn derives_spend_key_from_partials() {
    let alice = Scalar::from(2u64);
    let bob = Scalar::from(3u64);
    let secret = Scalar::from(5u64);
    let derived = derive_spend_key(&alice, &bob, &secret);
    assert_eq!(derived, alice + bob + secret);
}

#[test]
fn rejects_small_order_point() {
    let identity = EdwardsPoint::identity().compress().to_bytes();
    let result = validate_point(&identity);
    assert!(result.is_err());
}

#[test]
fn rejects_invalid_point_encoding() {
    // y = p (2^255 - 19) is non-canonical and must be rejected.
    let mut invalid = [0xffu8; 32];
    invalid[0] = 0xed;
    invalid[31] = 0x7f;
    let result = validate_point(&invalid);
    assert!(result.is_err());
}

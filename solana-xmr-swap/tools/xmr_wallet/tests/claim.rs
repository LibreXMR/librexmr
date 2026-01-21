use curve25519_dalek::scalar::Scalar;
use monero::Network;
use sha2::{Digest, Sha256};
use std::env;
use std::path::PathBuf;
use zeroize::Zeroizing;
use xmr_wallet::{
    derive_claim_keys, prepare_claim_guard, validate_destination_address, validate_hashlock,
    ClaimParams,
};

#[test]
fn derives_claim_keys_for_network() {
    let params = ClaimParams {
        alice_partial: Scalar::from(2u64),
        bob_partial: Scalar::from(3u64),
        revealed_secret: Zeroizing::new([5u8; 32]),
        destination_address: "stagenet_placeholder".to_string(),
        network: Network::Stagenet,
        wallet_filename: "claim_wallet".to_string(),
        restore_height: Some(0),
        expected_hashlock: None,
        claim_guard_path: None,
    };

    let keys = derive_claim_keys(&params).expect("derive claim keys");
    assert_eq!(keys.address.network, Network::Stagenet);
}

#[test]
fn rejects_invalid_destination_address() {
    let result = validate_destination_address("not-a-real-monero-address");
    assert!(result.is_err());
}

#[test]
fn rejects_wrong_secret_for_hashlock() {
    let secret = [1u8; 32];
    let expected: [u8; 32] = Sha256::digest([2u8; 32]).into();
    let result = validate_hashlock(&secret, &expected);
    assert!(result.is_err());
}

#[test]
fn prevents_replay_with_guard_file() {
    let mut path: PathBuf = env::temp_dir();
    path.push(format!("xmr_claim_guard_{}.lock", std::process::id()));
    let guard = prepare_claim_guard(&path).expect("create guard");
    guard.commit();
    let second = prepare_claim_guard(&path);
    assert!(second.is_err());
    let _ = std::fs::remove_file(&path);
}

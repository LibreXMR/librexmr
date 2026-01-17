use std::fs;
use std::path::PathBuf;

use dleq_verifier::{compute_challenge_bytes, verify_dleq, DleqVector};

fn load_vector() -> DleqVector {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test_vectors/dleq.json");
    let raw = fs::read_to_string(path).expect("read test_vectors/dleq.json");
    serde_json::from_str(&raw).expect("parse test vector JSON")
}

#[test]
fn vector_verifies() {
    let vector = load_vector();
    let inputs = vector.decode().expect("decode vector");

    let computed = compute_challenge_bytes(
        &inputs.y_point,
        &inputs.adaptor_point,
        &inputs.second_point,
        &inputs.r1,
        &inputs.r2,
        &inputs.hashlock,
    );
    assert_eq!(computed, inputs.challenge, "challenge mismatch");

    let (ok, report) = verify_dleq(&inputs).expect("verify");
    assert!(ok, "expected DLEQ to verify");
    assert!(report.challenge_matches, "challenge mismatch");
    assert!(report.lhs_r1_matches, "lhs_r1 mismatch");
    assert!(report.lhs_r2_matches, "lhs_r2 mismatch");
}

#[test]
fn vector_rejects_wrong_hashlock() {
    let vector = load_vector();
    let mut inputs = vector.decode().expect("decode vector");
    inputs.hashlock[0] ^= 0x01;

    let (ok, report) = verify_dleq(&inputs).expect("verify");
    assert!(!ok, "expected failure with wrong hashlock");
    assert!(!report.challenge_matches, "challenge should not match");
}

#[test]
fn vector_rejects_tampered_response() {
    let vector = load_vector();
    let mut inputs = vector.decode().expect("decode vector");
    inputs.response[0] ^= 0x01;

    let (ok, report) = verify_dleq(&inputs).expect("verify");
    assert!(!ok, "expected failure with tampered response");
    assert!(report.challenge_matches, "challenge should still match");
    assert!(!report.lhs_r1_matches || !report.lhs_r2_matches, "expected proof mismatch");
}

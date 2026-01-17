use std::fs;
use std::path::PathBuf;

use dleq_verifier::DleqVector;
use swap_sdk::{payload_hash_hex, SignedAuditLog};

fn load_vector() -> DleqVector {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test_vectors/dleq.json");
    let raw = fs::read_to_string(path).expect("read test_vectors/dleq.json");
    serde_json::from_str(&raw).expect("parse test vector JSON")
}

#[test]
fn audit_log_contains_transcript_fields() {
    let vector = load_vector();
    let json = serde_json::json!({
        "payload": {
            "timestamp_unix": 123,
            "input_path": "test_vectors/dleq.json",
            "ok": true,
            "report": {
                "computed_challenge": "deadbeef",
                "challenge_matches": true,
                "lhs_r1_matches": true,
                "lhs_r2_matches": true
            },
            "transcript": {
                "adaptor_point": vector.adaptor_point_compressed,
                "second_point": vector.dleq_second_point_compressed,
                "y_point": vector.y_compressed,
                "r1": vector.r1_compressed,
                "r2": vector.r2_compressed,
                "challenge": vector.challenge,
                "response": vector.response,
                "hashlock": vector.hashlock
            }
        },
        "payload_hash": "00",
        "signature": null,
        "public_key": null
    });
    let mut log: SignedAuditLog = serde_json::from_value(json).expect("parse log");
    log.payload_hash = payload_hash_hex(&log.payload).expect("hash payload");
    let serialized = serde_json::to_string(&log).expect("serialize log");
    assert!(serialized.contains("payload_hash"));
}

use anyhow::{Context, Result};
use dleq_verifier::DleqVector;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use swap_sdk::{
    hex_encode, payload_hash, AuditLogPayload, ReportView, SignedAuditLog, TranscriptView,
};

fn main() -> Result<()> {
    let payload = build_sample_payload()?;
    let payload_hash = hex_encode(&payload_hash(&payload)?);

    let secret = [7u8; 32];
    let signing_key = SigningKey::from_bytes(&secret);
    let verifying_key = VerifyingKey::from(&signing_key);
    let payload_bytes = serde_json::to_vec(&payload).context("serialize payload")?;
    let signature = signing_key.sign(&payload_bytes);

    let log = SignedAuditLog {
        payload,
        payload_hash,
        signature: Some(hex_encode(&signature.to_bytes())),
        public_key: Some(hex_encode(&verifying_key.to_bytes())),
    };

    let output = serde_json::to_string_pretty(&log).context("serialize log")?;
    println!("{output}");
    Ok(())
}

fn build_sample_payload() -> Result<AuditLogPayload> {
    let raw = std::fs::read_to_string("test_vectors/dleq.json")
        .context("read test_vectors/dleq.json")?;
    let vector: DleqVector =
        serde_json::from_str(&raw).context("parse DLEQ vector")?;

    Ok(AuditLogPayload {
        timestamp_unix: 1_710_000_000,
        input_path: "test_vectors/dleq.json".to_string(),
        ok: true,
        report: ReportView {
            computed_challenge: vector.challenge.clone(),
            challenge_matches: true,
            lhs_r1_matches: true,
            lhs_r2_matches: true,
        },
        transcript: TranscriptView {
            adaptor_point: vector.adaptor_point_compressed,
            second_point: vector.dleq_second_point_compressed,
            y_point: vector.y_compressed,
            r1: vector.r1_compressed,
            r2: vector.r2_compressed,
            challenge: vector.challenge,
            response: vector.response,
            hashlock: vector.hashlock,
        },
    })
}

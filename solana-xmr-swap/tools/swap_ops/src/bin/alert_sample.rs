use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Serialize)]
struct AlertPayload {
    event: String,
    pda: String,
    dleq_verified: bool,
    unlocked: bool,
    expired: bool,
    now_unix: i64,
    lock_until: i64,
}

#[derive(Serialize)]
struct AlertEnvelope {
    event: String,
    pda: String,
    dleq_verified: bool,
    unlocked: bool,
    expired: bool,
    now_unix: i64,
    lock_until: i64,
    payload_hash: Option<String>,
    signature: Option<String>,
    public_key: Option<String>,
}

fn main() {
    let payload = AlertPayload {
        event: "state_change".to_string(),
        pda: "11111111111111111111111111111111".to_string(),
        dleq_verified: true,
        unlocked: false,
        expired: false,
        now_unix: 1_710_000_000,
        lock_until: 1_710_003_600,
    };

    let payload_bytes = serde_json::to_vec(&payload).expect("serialize payload");
    let payload_hash = sha256_hex(&payload_bytes);

    let secret = [7u8; 32];
    let signing_key = SigningKey::from_bytes(&secret);
    let verifying_key = VerifyingKey::from(&signing_key);
    let signature = signing_key.sign(&payload_bytes);

    let envelope = AlertEnvelope {
        event: payload.event,
        pda: payload.pda,
        dleq_verified: payload.dleq_verified,
        unlocked: payload.unlocked,
        expired: payload.expired,
        now_unix: payload.now_unix,
        lock_until: payload.lock_until,
        payload_hash: Some(payload_hash),
        signature: Some(hex_encode(&signature.to_bytes())),
        public_key: Some(hex_encode(&verifying_key.to_bytes())),
    };

    let output = serde_json::to_string_pretty(&envelope).expect("serialize envelope");
    println!("{output}");
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex_encode(&digest)
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

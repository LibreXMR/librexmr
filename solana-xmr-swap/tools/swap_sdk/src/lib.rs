use anyhow::{anyhow, Context, Result};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Clone)]
pub struct SignedAuditLog {
    pub payload: AuditLogPayload,
    pub payload_hash: String,
    pub signature: Option<String>,
    pub public_key: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AuditLogPayload {
    pub timestamp_unix: i64,
    pub input_path: String,
    pub ok: bool,
    pub report: ReportView,
    pub transcript: TranscriptView,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReportView {
    pub computed_challenge: String,
    pub challenge_matches: bool,
    pub lhs_r1_matches: bool,
    pub lhs_r2_matches: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TranscriptView {
    pub adaptor_point: String,
    pub second_point: String,
    pub y_point: String,
    pub r1: String,
    pub r2: String,
    pub challenge: String,
    pub response: String,
    pub hashlock: String,
}

pub fn payload_hash(payload: &AuditLogPayload) -> Result<[u8; 32]> {
    let payload_bytes = serde_json::to_vec(payload).context("serialize payload")?;
    let digest = Sha256::digest(&payload_bytes);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    Ok(out)
}

pub fn payload_hash_hex(payload: &AuditLogPayload) -> Result<String> {
    Ok(hex_encode(&payload_hash(payload)?))
}

pub fn verify_signature(
    payload: &AuditLogPayload,
    signature_hex: &str,
    public_key_hex: &str,
) -> Result<()> {
    let signature_bytes = decode_hex_64(signature_hex)?;
    let public_key_bytes = decode_hex_32(public_key_hex)?;
    let signature = Signature::from_bytes(&signature_bytes);
    let verifying_key =
        VerifyingKey::from_bytes(&public_key_bytes).context("invalid public key")?;
    let payload_bytes = serde_json::to_vec(payload).context("serialize payload")?;
    verifying_key
        .verify(&payload_bytes, &signature)
        .context("signature verification failed")
}

pub fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

pub fn decode_hex_32(value: &str) -> Result<[u8; 32]> {
    let bytes = decode_hex(value)?;
    if bytes.len() != 32 {
        return Err(anyhow!("expected 32-byte hex, got {}", bytes.len()));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

pub fn decode_hex_64(value: &str) -> Result<[u8; 64]> {
    let bytes = decode_hex(value)?;
    if bytes.len() != 64 {
        return Err(anyhow!("expected 64-byte hex, got {}", bytes.len()));
    }
    let mut out = [0u8; 64];
    out.copy_from_slice(&bytes);
    Ok(out)
}

pub fn decode_hex(value: &str) -> Result<Vec<u8>> {
    if value.len() % 2 != 0 {
        return Err(anyhow!("hex string has odd length"));
    }
    let mut out = Vec::with_capacity(value.len() / 2);
    let mut iter = value.as_bytes().chunks_exact(2);
    while let Some(pair) = iter.next() {
        let hi = from_hex_digit(pair[0])?;
        let lo = from_hex_digit(pair[1])?;
        out.push((hi << 4) | lo);
    }
    Ok(out)
}

fn from_hex_digit(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(anyhow!("invalid hex digit: 0x{byte:02x}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    #[test]
    fn payload_hash_is_stable() {
        let payload = AuditLogPayload {
            timestamp_unix: 123,
            input_path: "path".to_string(),
            ok: true,
            report: ReportView {
                computed_challenge: "deadbeef".to_string(),
                challenge_matches: true,
                lhs_r1_matches: true,
                lhs_r2_matches: true,
            },
            transcript: TranscriptView {
                adaptor_point: "a".repeat(64),
                second_point: "b".repeat(64),
                y_point: "c".repeat(64),
                r1: "d".repeat(64),
                r2: "e".repeat(64),
                challenge: "f".repeat(64),
                response: "0".repeat(64),
                hashlock: "1".repeat(64),
            },
        };
        let first = payload_hash_hex(&payload).unwrap();
        let second = payload_hash_hex(&payload).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn signature_verifies_for_payload() {
        let payload = AuditLogPayload {
            timestamp_unix: 123,
            input_path: "test_vectors/dleq.json".to_string(),
            ok: true,
            report: ReportView {
                computed_challenge: "deadbeef".to_string(),
                challenge_matches: true,
                lhs_r1_matches: true,
                lhs_r2_matches: true,
            },
            transcript: TranscriptView {
                adaptor_point: "a".repeat(64),
                second_point: "b".repeat(64),
                y_point: "c".repeat(64),
                r1: "d".repeat(64),
                r2: "e".repeat(64),
                challenge: "f".repeat(64),
                response: "0".repeat(64),
                hashlock: "1".repeat(64),
            },
        };

        let secret = [7u8; 32];
        let signing_key = SigningKey::from_bytes(&secret);
        let verifying_key = VerifyingKey::from(&signing_key);
        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        let signature = signing_key.sign(&payload_bytes);
        let signature_hex = hex_encode(&signature.to_bytes());
        let public_key_hex = hex_encode(&verifying_key.to_bytes());

        verify_signature(&payload, &signature_hex, &public_key_hex).unwrap();
    }
}

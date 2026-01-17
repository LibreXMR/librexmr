use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Parser)]
#[command(name = "alert-verify", version, about = "Verify signed webhook alerts")]
struct Cli {
    /// Input alert JSON
    #[arg(long)]
    input: PathBuf,
    /// Allow unsigned alerts (skip signature requirement)
    #[arg(long, default_value_t = false)]
    allow_unsigned: bool,
}

#[derive(Deserialize)]
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

fn main() -> Result<()> {
    let args = Cli::parse();
    let raw = fs::read_to_string(&args.input)
        .with_context(|| format!("read {}", args.input.display()))?;
    let envelope: AlertEnvelope =
        serde_json::from_str(&raw).context("parse alert JSON")?;

    verify_alert(&envelope, args.allow_unsigned)?;
    println!("alert verification: OK");
    Ok(())
}

fn verify_alert(envelope: &AlertEnvelope, allow_unsigned: bool) -> Result<()> {
    let payload = AlertPayload {
        event: envelope.event.clone(),
        pda: envelope.pda.clone(),
        dleq_verified: envelope.dleq_verified,
        unlocked: envelope.unlocked,
        expired: envelope.expired,
        now_unix: envelope.now_unix,
        lock_until: envelope.lock_until,
    };
    let payload_bytes = serde_json::to_vec(&payload).context("serialize payload")?;
    let computed_hash = sha256_hex(&payload_bytes);

    match &envelope.payload_hash {
        Some(value) if value != &computed_hash => {
            return Err(anyhow!("payload_hash mismatch"));
        }
        Some(_) => {}
        None => {
            if envelope.signature.is_some() || envelope.public_key.is_some() {
                return Err(anyhow!("payload_hash missing"));
            }
        }
    }

    match (&envelope.signature, &envelope.public_key) {
        (Some(sig), Some(pk)) => {
            let signature = Signature::from_bytes(&decode_hex_64(sig)?);
            let verifying_key =
                VerifyingKey::from_bytes(&decode_hex_32(pk)?).context("invalid public key")?;
            verifying_key
                .verify(&payload_bytes, &signature)
                .context("signature verification failed")?;
        }
        (None, None) if allow_unsigned => {}
        (None, None) => return Err(anyhow!("signature/public_key missing")),
        _ => return Err(anyhow!("signature/public_key mismatch")),
    }

    Ok(())
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

fn decode_hex_32(value: &str) -> Result<[u8; 32]> {
    let bytes = decode_hex(value)?;
    if bytes.len() != 32 {
        return Err(anyhow!("expected 32-byte hex, got {}", bytes.len()));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn decode_hex_64(value: &str) -> Result<[u8; 64]> {
    let bytes = decode_hex(value)?;
    if bytes.len() != 64 {
        return Err(anyhow!("expected 64-byte hex, got {}", bytes.len()));
    }
    let mut out = [0u8; 64];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn decode_hex(value: &str) -> Result<Vec<u8>> {
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
    fn verify_signed_alert_roundtrip() {
        let payload = AlertPayload {
            event: "state_change".to_string(),
            pda: "pda".to_string(),
            dleq_verified: true,
            unlocked: false,
            expired: false,
            now_unix: 123,
            lock_until: 456,
        };
        let payload_bytes = serde_json::to_vec(&payload).unwrap();
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

        verify_alert(&envelope, false).unwrap();
    }
}

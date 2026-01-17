use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};

pub fn compute_hashlock_hex(secret_hex: &str) -> Result<String> {
    let secret_bytes = decode_hex(secret_hex, "secret")?;
    if secret_bytes.len() != 32 {
        return Err(anyhow!(
            "invalid secret length: expected 32 bytes, got {}",
            secret_bytes.len()
        ));
    }
    let hash = Sha256::digest(&secret_bytes);
    Ok(hex_encode(&hash))
}

pub fn verify_hashlock(secret_hex: &str, hashlock_hex: &str) -> Result<bool> {
    let expected_hash = decode_hex(hashlock_hex, "hashlock")?;
    if expected_hash.len() != 32 {
        return Err(anyhow!(
            "invalid hashlock length: expected 32 bytes, got {}",
            expected_hash.len()
        ));
    }
    let computed = decode_hex(&compute_hashlock_hex(secret_hex)?, "computed_hashlock")?;
    Ok(computed == expected_hash)
}

fn decode_hex(value: &str, field: &str) -> Result<Vec<u8>> {
    let normalized = value.trim().strip_prefix("0x").unwrap_or(value.trim());
    if normalized.len() % 2 != 0 {
        return Err(anyhow!("invalid hex for {field}: odd length"));
    }
    let mut out = Vec::with_capacity(normalized.len() / 2);
    let mut iter = normalized.as_bytes().chunks_exact(2);
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

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

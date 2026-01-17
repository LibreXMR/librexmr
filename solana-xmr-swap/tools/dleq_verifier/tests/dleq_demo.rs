use serde::Deserialize;

use dleq_verifier::{verify_dleq, DleqVector};
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
struct DemoVector {
    secret: String,
    #[serde(flatten)]
    vector: DleqVector,
}

#[test]
fn demo_vector_verifies() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("../../test_vectors/dleq_demo.json");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("read demo vector at {}", path.display()));
    let demo: DemoVector = serde_json::from_str(&raw).expect("parse demo vector");

    let secret = decode_hex_32(&demo.secret);
    let hashlock: [u8; 32] = Sha256::digest(secret).into();
    assert_eq!(hex_encode(hashlock), demo.vector.hashlock);

    let inputs = demo.vector.decode().expect("decode vector");
    let (ok, _) = verify_dleq(&inputs).expect("verify");
    assert!(ok, "demo vector should verify");
}

fn decode_hex_32(value: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    let bytes = decode_hex(value);
    assert_eq!(bytes.len(), 32);
    out.copy_from_slice(&bytes);
    out
}

fn decode_hex(value: &str) -> Vec<u8> {
    assert!(value.len() % 2 == 0);
    let mut out = Vec::with_capacity(value.len() / 2);
    let mut iter = value.as_bytes().chunks_exact(2);
    while let Some(pair) = iter.next() {
        let hi = from_hex_digit(pair[0]);
        let lo = from_hex_digit(pair[1]);
        out.push((hi << 4) | lo);
    }
    out
}

fn from_hex_digit(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => panic!("invalid hex digit: {byte}"),
    }
}

fn hex_encode(bytes: [u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(64);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

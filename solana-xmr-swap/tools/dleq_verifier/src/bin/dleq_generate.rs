use std::fs;
use std::path::PathBuf;

use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
use curve25519_dalek::scalar::Scalar;
use dleq_verifier::compute_challenge_bytes;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Serialize)]
struct DleqDemoVector {
    adaptor_point_compressed: String,
    challenge: String,
    dleq_second_point_compressed: String,
    hashlock: String,
    r1_compressed: String,
    r2_compressed: String,
    response: String,
    y_compressed: String,
    secret: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut output = PathBuf::from("test_vectors/dleq_demo.json");
    let mut stdout = false;
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => {
                let value = args.next().ok_or("missing --output value")?;
                output = PathBuf::from(value);
            }
            "--stdout" => stdout = true,
            _ => {
                print_usage();
                std::process::exit(2);
            }
        }
    }

    let vector = generate_demo_vector();
    let json = serde_json::to_string_pretty(&vector)?;

    if stdout {
        println!("{json}");
        return Ok(());
    }

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output, json)?;
    println!("wrote demo vector: {}", output.display());
    Ok(())
}

fn generate_demo_vector() -> DleqDemoVector {
    let mut rng = OsRng;
    let t = Scalar::from_bytes_mod_order(random_bytes(&mut rng));
    let k = Scalar::from_bytes_mod_order(random_bytes(&mut rng));
    let y = Scalar::from_bytes_mod_order(random_bytes(&mut rng));

    let y_point = &ED25519_BASEPOINT_POINT * &y;
    let t_point = &ED25519_BASEPOINT_POINT * &t;
    let u_point = &y_point * &t;

    let r1_point = &ED25519_BASEPOINT_POINT * &k;
    let r2_point = &y_point * &k;

    let secret_bytes = t.to_bytes();
    let hashlock: [u8; 32] = Sha256::digest(secret_bytes).into();
    let challenge = compute_challenge_bytes(
        &y_point.compress().to_bytes(),
        &t_point.compress().to_bytes(),
        &u_point.compress().to_bytes(),
        &r1_point.compress().to_bytes(),
        &r2_point.compress().to_bytes(),
        &hashlock,
    );

    let c = Scalar::from_bytes_mod_order(challenge);
    let s = k + c * t;

    DleqDemoVector {
        adaptor_point_compressed: hex_encode(t_point.compress().to_bytes()),
        challenge: hex_encode(challenge),
        dleq_second_point_compressed: hex_encode(u_point.compress().to_bytes()),
        hashlock: hex_encode(hashlock),
        r1_compressed: hex_encode(r1_point.compress().to_bytes()),
        r2_compressed: hex_encode(r2_point.compress().to_bytes()),
        response: hex_encode(s.to_bytes()),
        y_compressed: hex_encode(y_point.compress().to_bytes()),
        secret: hex_encode(secret_bytes),
    }
}

fn print_usage() {
    eprintln!("Usage: dleq-generate [--output <path>] [--stdout]");
    eprintln!("Default output: test_vectors/dleq_demo.json");
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

fn random_bytes(rng: &mut OsRng) -> [u8; 32] {
    let mut out = [0u8; 32];
    rng.fill_bytes(&mut out);
    out
}

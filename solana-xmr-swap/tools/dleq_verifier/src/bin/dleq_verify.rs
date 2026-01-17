use std::fs;
use std::path::PathBuf;

use dleq_verifier::{verify_dleq, DleqVector};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mut input = None;
    let mut verbose = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => {
                input = args.next().map(PathBuf::from);
            }
            "--verbose" => verbose = true,
            _ => {
                eprintln!("Unknown arg: {arg}");
                print_usage();
                std::process::exit(2);
            }
        }
    }

    let input = input.unwrap_or_else(|| PathBuf::from("test_vectors/dleq.json"));
    let raw = fs::read_to_string(&input)?;
    let vector: DleqVector = serde_json::from_str(&raw)?;
    let inputs = vector.decode()?;

    let (ok, report) = verify_dleq(&inputs)?;
    if verbose {
        println!("computed_challenge: {}", hex_encode(report.computed_challenge));
        println!("challenge_matches: {}", report.challenge_matches);
        println!("lhs_r1_matches: {}", report.lhs_r1_matches);
        println!("lhs_r2_matches: {}", report.lhs_r2_matches);
    }

    if ok {
        println!("DLEQ verification: OK");
        Ok(())
    } else {
        eprintln!("DLEQ verification: FAILED");
        std::process::exit(1);
    }
}

fn print_usage() {
    eprintln!("Usage: dleq-verify [--input <path>] [--verbose]");
    eprintln!("Default input: test_vectors/dleq.json");
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

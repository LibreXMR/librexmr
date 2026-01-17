use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use swap_sdk::{payload_hash_hex, verify_signature, SignedAuditLog};

#[derive(Parser)]
#[command(name = "audit-verify", version, about = "Verify signed audit logs")]
struct Cli {
    /// Input audit log JSON
    #[arg(long)]
    input: PathBuf,
    /// Allow unsigned logs (skip signature requirement)
    #[arg(long, default_value_t = false)]
    allow_unsigned: bool,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let raw = fs::read_to_string(&args.input)
        .with_context(|| format!("read {}", args.input.display()))?;
    let log: SignedAuditLog =
        serde_json::from_str(&raw).context("parse audit log JSON")?;

    let computed_hash = payload_hash_hex(&log.payload)?;
    if computed_hash != log.payload_hash {
        return Err(anyhow!("payload_hash mismatch"));
    }

    match (&log.signature, &log.public_key) {
        (Some(sig), Some(pk)) => {
            verify_signature(&log.payload, sig, pk)?;
        }
        (None, None) if args.allow_unsigned => {}
        _ => {
            return Err(anyhow!("signature/public_key missing"));
        }
    }

    println!("audit log verification: OK");
    Ok(())
}

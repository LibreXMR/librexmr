use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use dleq_verifier::{verify_dleq, DleqReport, DleqVector};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use swap_sdk::{
    decode_hex_32, hex_encode, payload_hash, AuditLogPayload, ReportView, SignedAuditLog,
    TranscriptView,
};

#[derive(Parser)]
#[command(name = "swap-sdk", version, about = "Client-side swap verification tooling")]
struct Cli {
    /// Path to DLEQ vector JSON (default: test_vectors/dleq.json)
    #[arg(long)]
    input: Option<PathBuf>,
    /// Output audit log path (default: audit_logs/<timestamp>_dleq.json)
    #[arg(long)]
    output: Option<PathBuf>,
    /// Write audit log to stdout only
    #[arg(long, default_value_t = false)]
    stdout: bool,
    /// Do not require DLEQ verification to pass (debug only)
    #[arg(long, default_value_t = false)]
    allow_fail: bool,
    /// Signing key in hex (32 bytes)
    #[arg(long)]
    signing_key: Option<String>,
    /// Signing key file containing hex (32 bytes)
    #[arg(long)]
    signing_key_file: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let input_path = args
        .input
        .clone()
        .unwrap_or_else(|| PathBuf::from("test_vectors/dleq.json"));
    let raw = fs::read_to_string(&input_path)
        .with_context(|| format!("read {}", input_path.display()))?;
    let vector: DleqVector =
        serde_json::from_str(&raw).context("parse DLEQ vector JSON")?;
    let inputs = vector.decode().context("decode DLEQ vector")?;

    let (ok, report) = verify_dleq(&inputs).context("verify DLEQ")?;
    let payload = AuditLogPayload {
        timestamp_unix: current_unix_timestamp(),
        input_path: input_path.display().to_string(),
        ok,
        report: report_view(&report),
        transcript: transcript_view(&vector),
    };

    let (signature, public_key) = sign_payload(&payload, &args)?;
    let payload_hash = hex_encode(&payload_hash(&payload)?);

    let log = SignedAuditLog {
        payload,
        payload_hash,
        signature,
        public_key,
    };

    let out = serde_json::to_string_pretty(&log).context("serialize audit log")?;
    if args.stdout {
        println!("{out}");
    } else {
        let output = args
            .output
            .unwrap_or_else(|| default_output_path(log.payload.timestamp_unix));
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(&output, out)
            .with_context(|| format!("write {}", output.display()))?;
        println!("wrote audit log: {}", output.display());
    }

    if !ok && !args.allow_fail {
        return Err(anyhow!("DLEQ verification failed"));
    }

    Ok(())
}

fn report_view(report: &DleqReport) -> ReportView {
    ReportView {
        computed_challenge: hex_encode(&report.computed_challenge),
        challenge_matches: report.challenge_matches,
        lhs_r1_matches: report.lhs_r1_matches,
        lhs_r2_matches: report.lhs_r2_matches,
    }
}

fn transcript_view(vector: &DleqVector) -> TranscriptView {
    TranscriptView {
        adaptor_point: vector.adaptor_point_compressed.clone(),
        second_point: vector.dleq_second_point_compressed.clone(),
        y_point: vector.y_compressed.clone(),
        r1: vector.r1_compressed.clone(),
        r2: vector.r2_compressed.clone(),
        challenge: vector.challenge.clone(),
        response: vector.response.clone(),
        hashlock: vector.hashlock.clone(),
    }
}

fn default_output_path(timestamp: i64) -> PathBuf {
    PathBuf::from("audit_logs").join(format!("{timestamp}_dleq.json"))
}

fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn sign_payload(payload: &AuditLogPayload, args: &Cli) -> Result<(Option<String>, Option<String>)> {
    let key_hex = match (&args.signing_key, &args.signing_key_file) {
        (Some(_), Some(_)) => return Err(anyhow!("provide only one signing key source")),
        (Some(value), None) => Some(value.clone()),
        (None, Some(path)) => {
            let raw = fs::read_to_string(path)
                .with_context(|| format!("read {}", path.display()))?;
            Some(raw.trim().to_string())
        }
        (None, None) => None,
    };

    let Some(key_hex) = key_hex else {
        return Ok((None, None));
    };

    let secret = decode_hex_32(&key_hex)?;
    let signing_key = SigningKey::from_bytes(&secret);
    let verifying_key = VerifyingKey::from(&signing_key);
    let payload_bytes = serde_json::to_vec(payload).context("serialize payload")?;
    let signature: Signature = signing_key.sign(&payload_bytes);

    let signature_hex = hex_encode(&signature.to_bytes());
    let public_key_hex = hex_encode(&verifying_key.to_bytes());
    Ok((Some(signature_hex), Some(public_key_hex)))
}


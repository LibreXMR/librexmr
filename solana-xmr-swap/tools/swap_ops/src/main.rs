use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::Json;
use axum::Router;
use clap::{Parser, Subcommand};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use reqwest::Client;
use serde::Serialize;
use sha2::{Digest, Sha256};
use solana_client::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;

use atomic_lock::state::AtomicLock;
use anchor_lang::AccountDeserialize;

#[derive(Parser)]
#[command(name = "swap-ops", version, about = "Swap operations utilities")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Poll swap accounts and emit alerts on state changes.
    Watch(WatchArgs),
    /// Serve a basic status API for swap accounts.
    Serve(ServeArgs),
}

#[derive(Parser)]
struct WatchArgs {
    /// Solana RPC URL
    #[arg(long, default_value = "http://127.0.0.1:8899")]
    rpc: String,
    /// Swap PDA (repeatable)
    #[arg(long)]
    pda: Vec<String>,
    /// File containing one PDA per line
    #[arg(long)]
    pda_file: Option<PathBuf>,
    /// Poll interval in seconds
    #[arg(long, default_value_t = 10)]
    poll_secs: u64,
    /// Optional webhook URL for alerts
    #[arg(long)]
    webhook: Option<String>,
    /// Webhook timeout in seconds
    #[arg(long, default_value_t = 5)]
    webhook_timeout_secs: u64,
    /// Webhook retry attempts (in addition to the first try)
    #[arg(long, default_value_t = 2)]
    webhook_retries: u32,
    /// Webhook backoff base (ms)
    #[arg(long, default_value_t = 500)]
    webhook_backoff_ms: u64,
    /// Webhook signing key in hex (32 bytes)
    #[arg(long)]
    webhook_signing_key: Option<String>,
    /// Webhook signing key file containing hex (32 bytes)
    #[arg(long)]
    webhook_signing_key_file: Option<PathBuf>,
}

#[derive(Parser)]
struct ServeArgs {
    /// Solana RPC URL
    #[arg(long, default_value = "http://127.0.0.1:8899")]
    rpc: String,
    /// Bind address
    #[arg(long, default_value = "127.0.0.1:8080")]
    bind: String,
    /// Swap PDA (repeatable)
    #[arg(long)]
    pda: Vec<String>,
    /// File containing one PDA per line
    #[arg(long)]
    pda_file: Option<PathBuf>,
    /// Allow any PDA (disables whitelist)
    #[arg(long, default_value_t = false)]
    allow_any: bool,
}

#[derive(Clone)]
struct AppState {
    rpc: Arc<RpcClient>,
    whitelist: HashSet<Pubkey>,
    allow_any: bool,
}

#[derive(Serialize)]
struct SwapStatus {
    pda: String,
    depositor: String,
    hashlock: String,
    adaptor_point: String,
    second_point: String,
    y_point: String,
    r1: String,
    r2: String,
    challenge: String,
    response: String,
    lock_until: i64,
    amount: u64,
    token_mint: String,
    vault: String,
    dleq_verified: bool,
    unlocked: bool,
    now_unix: i64,
    expired: bool,
}

#[derive(Clone)]
struct WebhookSigner {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Watch(args) => watch(args).await,
        Command::Serve(args) => serve(args).await,
    }
}

async fn watch(args: WatchArgs) -> Result<()> {
    let rpc = RpcClient::new(args.rpc.clone());
    let pda_list = load_pdas(&args.pda, &args.pda_file)?;
    let signer = load_webhook_signer(&args)?;

    if pda_list.is_empty() {
        return Err(anyhow!("no PDAs provided"));
    }

    let mut last_state: HashMap<Pubkey, (bool, bool)> = HashMap::new();
    let webhook = build_webhook_client(&args);
    let mut interval = tokio::time::interval(Duration::from_secs(args.poll_secs));

    loop {
        interval.tick().await;
        for pda in &pda_list {
            match fetch_status(&rpc, pda) {
                Ok(status) => {
                    let key = (status.dleq_verified, status.unlocked);
                    let entry = last_state.entry(*pda).or_insert(key);
                    if *entry != key {
                        emit_alert(&webhook, &signer, "state_change", &status).await;
                        *entry = key;
                    }
                    if status.expired && !status.unlocked {
                        emit_alert(&webhook, &signer, "expired_unlocked_false", &status).await;
                    }
                }
                Err(err) => {
                    eprintln!("watch error for {}: {}", pda, err);
                }
            }
        }
    }
}

async fn serve(args: ServeArgs) -> Result<()> {
    let rpc = RpcClient::new(args.rpc);
    let whitelist = load_pdas(&args.pda, &args.pda_file)?;
    let bind: SocketAddr = args.bind.parse().context("invalid bind address")?;

    let state = AppState {
        rpc: Arc::new(rpc),
        whitelist,
        allow_any: args.allow_any,
    };

    let app = Router::new()
        .route("/health", get(|| async { StatusCode::OK }))
        .route("/swap/:pda", get(handle_swap))
        .with_state(state);

    axum::serve(tokio::net::TcpListener::bind(bind).await?, app).await?;
    Ok(())
}

async fn handle_swap(
    State(state): State<AppState>,
    Path(pda): Path<String>,
) -> Result<Json<SwapStatus>, StatusCode> {
    let pda = Pubkey::from_str(&pda).map_err(|_| StatusCode::BAD_REQUEST)?;
    if !state.allow_any && !state.whitelist.contains(&pda) {
        return Err(StatusCode::FORBIDDEN);
    }

    let status = fetch_status(&state.rpc, &pda).map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(status))
}

fn fetch_status(rpc: &RpcClient, pda: &Pubkey) -> Result<SwapStatus> {
    let account = rpc
        .get_account(pda)
        .with_context(|| format!("account not found: {pda}"))?;

    let lock = decode_lock(&account)?;
    let now_unix = current_unix_timestamp();
    let expired = now_unix >= lock.lock_until;

    Ok(SwapStatus {
        pda: pda.to_string(),
        depositor: lock.depositor.to_string(),
        hashlock: hex_encode(&lock.hashlock),
        adaptor_point: hex_encode(&lock.adaptor_point),
        second_point: hex_encode(&lock.second_point),
        y_point: hex_encode(&lock.y_point),
        r1: hex_encode(&lock.r1),
        r2: hex_encode(&lock.r2),
        challenge: hex_encode(&lock.challenge),
        response: hex_encode(&lock.response),
        lock_until: lock.lock_until,
        amount: lock.amount,
        token_mint: lock.token_mint.to_string(),
        vault: lock.vault.to_string(),
        dleq_verified: lock.dleq_verified,
        unlocked: lock.unlocked,
        now_unix,
        expired,
    })
}

fn decode_lock(account: &Account) -> Result<AtomicLock> {
    let mut data = account.data.as_slice();
    AtomicLock::try_deserialize(&mut data).context("failed to decode AtomicLock")
}

fn load_pdas(pda_args: &[String], pda_file: &Option<PathBuf>) -> Result<HashSet<Pubkey>> {
    let mut pdas = HashSet::new();
    for pda in pda_args {
        let pubkey = Pubkey::from_str(pda)
            .with_context(|| format!("invalid PDA: {pda}"))?;
        pdas.insert(pubkey);
    }

    if let Some(path) = pda_file {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {path:?}"))?;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let pubkey = Pubkey::from_str(trimmed)
                .with_context(|| format!("invalid PDA in file: {trimmed}"))?;
            pdas.insert(pubkey);
        }
    }

    Ok(pdas)
}

fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
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
    #[serde(skip_serializing_if = "Option::is_none")]
    payload_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    public_key: Option<String>,
}

pub(crate) fn build_alert_payload(
    kind: &str,
    status: &SwapStatus,
    signer: &Option<WebhookSigner>,
) -> Result<serde_json::Value> {
    let payload = AlertPayload {
        event: kind.to_string(),
        pda: status.pda.clone(),
        dleq_verified: status.dleq_verified,
        unlocked: status.unlocked,
        expired: status.expired,
        now_unix: status.now_unix,
        lock_until: status.lock_until,
    };

    let mut envelope = AlertEnvelope {
        event: payload.event.clone(),
        pda: payload.pda.clone(),
        dleq_verified: payload.dleq_verified,
        unlocked: payload.unlocked,
        expired: payload.expired,
        now_unix: payload.now_unix,
        lock_until: payload.lock_until,
        payload_hash: None,
        signature: None,
        public_key: None,
    };

    if let Some(signer) = signer {
        let payload_bytes = serde_json::to_vec(&payload).context("serialize alert payload")?;
        let payload_hash = sha256_hex(&payload_bytes);
        let signature: Signature = signer.signing_key.sign(&payload_bytes);
        envelope.payload_hash = Some(payload_hash);
        envelope.signature = Some(hex_encode(&signature.to_bytes()));
        envelope.public_key = Some(hex_encode(&signer.verifying_key.to_bytes()));
    }

    serde_json::to_value(&envelope).context("serialize alert envelope")
}

fn build_webhook_client(args: &WatchArgs) -> Option<WebhookClient> {
    let url = args.webhook.clone()?;
    let timeout = Duration::from_secs(args.webhook_timeout_secs);
    let client = Client::builder().timeout(timeout).build().ok()?;
    Some(WebhookClient {
        client,
        url,
        retries: args.webhook_retries,
        backoff_ms: args.webhook_backoff_ms,
    })
}

async fn emit_alert(
    webhook: &Option<WebhookClient>,
    signer: &Option<WebhookSigner>,
    kind: &str,
    status: &SwapStatus,
) {
    let payload = match build_alert_payload(kind, status, signer) {
        Ok(payload) => payload,
        Err(err) => {
            eprintln!("alert build error: {err}");
            return;
        }
    };
    println!("{}", payload);
    if let Some(webhook) = webhook {
        if let Err(err) = webhook.post(payload).await {
            eprintln!("webhook error: {err}");
        }
    }
}

struct WebhookClient {
    client: Client,
    url: String,
    retries: u32,
    backoff_ms: u64,
}

impl WebhookClient {
    async fn post(&self, payload: serde_json::Value) -> Result<()> {
        let mut attempt = 0u32;
        loop {
            match self.client.post(&self.url).json(&payload).send().await {
                Ok(resp) if resp.status().is_success() => return Ok(()),
                Ok(resp) => {
                    attempt += 1;
                    if attempt > self.retries {
                        return Err(anyhow!("webhook status {}", resp.status()));
                    }
                }
                Err(err) => {
                    attempt += 1;
                    if attempt > self.retries {
                        return Err(anyhow!("webhook error: {err}"));
                    }
                }
            }

            let delay = backoff_delay(self.backoff_ms, attempt);
            tokio::time::sleep(delay).await;
        }
    }
}

fn backoff_delay(base_ms: u64, attempt: u32) -> Duration {
    let factor = 1u64
        .checked_shl(attempt.min(31))
        .unwrap_or(u64::MAX);
    Duration::from_millis(base_ms.saturating_mul(factor))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex_encode(&digest)
}

fn load_webhook_signer(args: &WatchArgs) -> Result<Option<WebhookSigner>> {
    let key_hex = match (&args.webhook_signing_key, &args.webhook_signing_key_file) {
        (Some(_), Some(_)) => {
            return Err(anyhow!(
                "provide only one of --webhook-signing-key or --webhook-signing-key-file"
            ));
        }
        (Some(value), None) => Some(value.clone()),
        (None, Some(path)) => {
            let raw =
                std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
            Some(raw.trim().to_string())
        }
        (None, None) => None,
    };

    let Some(key_hex) = key_hex else {
        return Ok(None);
    };

    let secret = decode_hex_32(&key_hex)?;
    let signing_key = SigningKey::from_bytes(&secret);
    let verifying_key = VerifyingKey::from(&signing_key);
    Ok(Some(WebhookSigner {
        signing_key,
        verifying_key,
    }))
}

fn decode_hex_32(value: &str) -> Result<[u8; 32]> {
    let bytes = decode_hex(value)?;
    if bytes.len() != 32 {
        return Err(anyhow!("expected 32-byte hex key, got {}", bytes.len()));
    }
    let mut out = [0u8; 32];
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

    #[test]
    fn alert_payload_contains_expected_fields() {
        let status = SwapStatus {
            pda: "pda".to_string(),
            depositor: "depositor".to_string(),
            hashlock: "hashlock".to_string(),
            adaptor_point: "adaptor".to_string(),
            second_point: "second".to_string(),
            y_point: "y".to_string(),
            r1: "r1".to_string(),
            r2: "r2".to_string(),
            challenge: "challenge".to_string(),
            response: "response".to_string(),
            lock_until: 123,
            amount: 456,
            token_mint: "mint".to_string(),
            vault: "vault".to_string(),
            dleq_verified: true,
            unlocked: false,
            now_unix: 111,
            expired: false,
        };

        let payload = build_alert_payload("state_change", &status, &None).expect("payload");
        let obj = payload.as_object().expect("payload object");
        for key in [
            "event",
            "pda",
            "dleq_verified",
            "unlocked",
            "expired",
            "now_unix",
            "lock_until",
        ] {
            assert!(obj.contains_key(key), "missing {key}");
        }
    }

    #[test]
    fn signed_payload_includes_signature_fields() {
        let status = SwapStatus {
            pda: "pda".to_string(),
            depositor: "depositor".to_string(),
            hashlock: "hashlock".to_string(),
            adaptor_point: "adaptor".to_string(),
            second_point: "second".to_string(),
            y_point: "y".to_string(),
            r1: "r1".to_string(),
            r2: "r2".to_string(),
            challenge: "challenge".to_string(),
            response: "response".to_string(),
            lock_until: 123,
            amount: 456,
            token_mint: "mint".to_string(),
            vault: "vault".to_string(),
            dleq_verified: true,
            unlocked: false,
            now_unix: 111,
            expired: false,
        };

        let secret = [7u8; 32];
        let signing_key = SigningKey::from_bytes(&secret);
        let verifying_key = VerifyingKey::from(&signing_key);
        let signer = WebhookSigner {
            signing_key,
            verifying_key,
        };

        let payload =
            build_alert_payload("state_change", &status, &Some(signer)).expect("payload");
        let obj = payload.as_object().expect("payload object");
        for key in ["payload_hash", "signature", "public_key"] {
            assert!(obj.contains_key(key), "missing {key}");
            assert!(obj.get(key).is_some(), "empty {key}");
        }
    }

    #[test]
    fn backoff_is_exponential() {
        assert_eq!(backoff_delay(100, 0), Duration::from_millis(100));
        assert_eq!(backoff_delay(100, 1), Duration::from_millis(200));
        assert_eq!(backoff_delay(100, 2), Duration::from_millis(400));
    }
}

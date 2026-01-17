use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use xmr_simulator::MockWallet;

#[derive(Parser)]
#[command(name = "xmr-simulator", version, about = "Simulated Monero wallet flow")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Lock XMR funds against a hashlock (simulated)
    Lock(LockArgs),
    /// Claim XMR funds by revealing the secret (simulated)
    Claim(ClaimArgs),
}

#[derive(Parser)]
struct LockArgs {
    /// Swap ID to track the lock
    #[arg(long)]
    swap_id: String,
    /// Hashlock (SHA-256 of secret)
    #[arg(long)]
    hashlock: String,
    /// Amount in piconero
    #[arg(long)]
    amount: u64,
    /// Directory to store simulator state
    #[arg(long, default_value = "xmr_simulator_state")]
    state_dir: PathBuf,
}

#[derive(Parser)]
struct ClaimArgs {
    /// Swap ID to claim
    #[arg(long)]
    swap_id: String,
    /// Secret (hashlock preimage)
    #[arg(long)]
    secret: String,
    /// Directory to store simulator state
    #[arg(long, default_value = "xmr_simulator_state")]
    state_dir: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Lock(args) => lock_swap(args),
        Command::Claim(args) => claim_swap(args),
    }
}

fn lock_swap(args: LockArgs) -> Result<()> {
    let wallet = MockWallet::new(&args.state_dir)
        .with_context(|| format!("init simulator at {:?}", args.state_dir))?;
    let record = wallet.lock(&args.swap_id, &args.hashlock, args.amount, now_unix())?;
    println!("{}", serde_json::to_string_pretty(&record)?);
    Ok(())
}

fn claim_swap(args: ClaimArgs) -> Result<()> {
    let wallet = MockWallet::new(&args.state_dir)
        .with_context(|| format!("init simulator at {:?}", args.state_dir))?;
    let record = wallet.claim(&args.swap_id, &args.secret, now_unix())?;
    println!("{}", serde_json::to_string_pretty(&record)?);
    Ok(())
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

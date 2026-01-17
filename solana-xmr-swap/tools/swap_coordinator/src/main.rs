mod db;
mod driver;
mod solana;
mod state;

use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use rand::RngCore;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Keypair, Signer};
use solana_system_interface::instruction::create_account;
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::instruction as token_instruction;

use dleq_verifier::{verify_dleq, DleqVector};

use crate::db::{JsonFileDb, SwapDb};
use crate::driver::step;
use crate::solana::SolanaSwapClient;
use crate::state::SwapState;

#[derive(Parser)]
#[command(name = "swap-coordinator", version, about = "Swap orchestration CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run a full demo swap flow (initialize -> verify -> unlock)
    Demo(DemoArgs),
    /// Create a new mint and fund the depositor
    SetupMint(SetupMintArgs),
}

#[derive(Parser)]
struct DemoArgs {
    /// Solana RPC URL
    #[arg(long, default_value = "http://127.0.0.1:8899")]
    rpc: String,
    /// Program ID (atomic_lock)
    #[arg(long, default_value = "Fg6PaFpoGXkYsidMpWxqSWYv7jD6f9W5KqPrk9N5Y7vC")]
    program_id: String,
    /// Depositor keypair path
    #[arg(long)]
    depositor: PathBuf,
    /// Unlocker keypair path (defaults to depositor)
    #[arg(long)]
    unlocker: Option<PathBuf>,
    /// Token mint address (optional; will create if omitted)
    #[arg(long)]
    mint: Option<String>,
    /// Amount in base units
    #[arg(long)]
    amount: u64,
    /// Lock duration in seconds
    #[arg(long, default_value_t = 3600)]
    lock_duration: u64,
    /// DLEQ demo vector path
    #[arg(long, default_value = "test_vectors/dleq_demo.json")]
    vector: PathBuf,
    /// State directory
    #[arg(long, default_value = "swap_state")]
    state_dir: PathBuf,
    /// Optional airdrop lamports (localnet)
    #[arg(long)]
    airdrop: Option<u64>,
}

#[derive(Parser)]
struct SetupMintArgs {
    /// Solana RPC URL
    #[arg(long, default_value = "http://127.0.0.1:8899")]
    rpc: String,
    /// Depositor keypair path
    #[arg(long)]
    depositor: PathBuf,
    /// Amount to mint (base units)
    #[arg(long)]
    amount: u64,
    /// Mint decimals
    #[arg(long, default_value_t = 9)]
    decimals: u8,
}

#[derive(Deserialize)]
struct DemoVector {
    secret: String,
    #[serde(flatten)]
    vector: DleqVector,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Demo(args) => run_demo(args).await,
        Command::SetupMint(args) => run_setup_mint(args),
    }
}

async fn run_demo(args: DemoArgs) -> Result<()> {
    let depositor = read_keypair_file(&args.depositor)
        .map_err(|err| anyhow!("read depositor keypair {}: {}", args.depositor.display(), err))?;
    let unlocker = if let Some(path) = args.unlocker.as_ref() {
        read_keypair_file(path)
            .map_err(|err| anyhow!("read unlocker keypair {}: {}", path.display(), err))?
    } else {
        Keypair::try_from(&depositor.to_bytes()[..])?
    };

    let rpc = RpcClient::new_with_commitment(args.rpc.clone(), CommitmentConfig::confirmed());
    if let Some(lamports) = args.airdrop {
        request_airdrop(&rpc, depositor.pubkey(), lamports)?;
        if unlocker.pubkey() != depositor.pubkey() {
            request_airdrop(&rpc, unlocker.pubkey(), lamports)?;
        }
    }

    let demo = load_demo_vector(&args.vector)?;
    validate_demo_vector(&demo)?;

    let program_id = Pubkey::from_str(&args.program_id)
        .context("invalid program id")?;

    let token_mint = if let Some(mint) = args.mint.as_ref() {
        Pubkey::from_str(mint).context("invalid mint address")?
    } else {
        create_mint_and_fund(
            &rpc,
            &depositor,
            args.amount,
            9,
        )?
    };

    let swap_id = generate_swap_id();
    let state = SwapState::Created {
        swap_id: swap_id.clone(),
        depositor: depositor.pubkey().to_string(),
        unlocker: unlocker.pubkey().to_string(),
        token_mint: token_mint.to_string(),
        amount: args.amount,
        lock_duration_secs: args.lock_duration,
        hashlock: decode_hex_32(&demo.vector.hashlock)?,
        adaptor_point: decode_hex_32(&demo.vector.adaptor_point_compressed)?,
        second_point: decode_hex_32(&demo.vector.dleq_second_point_compressed)?,
        y_point: decode_hex_32(&demo.vector.y_compressed)?,
        r1: decode_hex_32(&demo.vector.r1_compressed)?,
        r2: decode_hex_32(&demo.vector.r2_compressed)?,
        challenge: decode_hex_32(&demo.vector.challenge)?,
        response: decode_hex_32(&demo.vector.response)?,
    };

    let db = JsonFileDb::new(&args.state_dir)?;
    db.save(&state)?;

    let mut current = state;
    let secret = decode_hex_32(&demo.secret)?;

    let client = SolanaSwapClient::new(
        &args.rpc,
        program_id,
        depositor,
        unlocker,
        token_mint,
        args.amount,
        decode_hex_32(&demo.vector.hashlock)?,
        decode_hex_32(&demo.vector.adaptor_point_compressed)?,
        decode_hex_32(&demo.vector.dleq_second_point_compressed)?,
        decode_hex_32(&demo.vector.y_compressed)?,
        decode_hex_32(&demo.vector.r1_compressed)?,
        decode_hex_32(&demo.vector.r2_compressed)?,
        decode_hex_32(&demo.vector.challenge)?,
        decode_hex_32(&demo.vector.response)?,
    );

    println!("swap_id: {swap_id}");
    println!("depositor: {}", client.depositor_pubkey());
    println!("unlocker: {}", client.unlocker_pubkey());
    println!("token_mint: {token_mint}");

    loop {
        if current.is_terminal() {
            break;
        }
        match step(&current, &db, &client, Some(secret)).await? {
            Some(next) => {
                println!("state -> {:?}", next);
                current = next;
            }
            None => break,
        }
    }

    Ok(())
}

fn run_setup_mint(args: SetupMintArgs) -> Result<()> {
    let depositor = read_keypair_file(&args.depositor)
        .map_err(|err| anyhow!("read depositor keypair {}: {}", args.depositor.display(), err))?;
    let rpc = RpcClient::new_with_commitment(args.rpc.clone(), CommitmentConfig::confirmed());
    let mint = create_mint_and_fund(&rpc, &depositor, args.amount, args.decimals)?;
    println!("mint: {mint}");
    Ok(())
}

fn load_demo_vector(path: &PathBuf) -> Result<DemoVector> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read vector {}", path.display()))?;
    let demo: DemoVector = serde_json::from_str(&raw).context("parse demo vector")?;
    Ok(demo)
}

fn validate_demo_vector(demo: &DemoVector) -> Result<()> {
    let inputs = demo.vector.decode().context("decode vector")?;
    let (ok, report) = verify_dleq(&inputs).context("verify DLEQ")?;
    if !ok {
        return Err(anyhow!(
            "DLEQ verification failed (challenge_matches={}, lhs_r1={}, lhs_r2={})",
            report.challenge_matches,
            report.lhs_r1_matches,
            report.lhs_r2_matches
        ));
    }

    let secret = decode_hex_32(&demo.secret)?;
    let computed_hashlock: [u8; 32] = Sha256::digest(secret).into();
    let expected_hashlock = decode_hex_32(&demo.vector.hashlock)?;
    if computed_hashlock != expected_hashlock {
        return Err(anyhow!("secret/hashlock mismatch"));
    }
    Ok(())
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

fn generate_swap_id() -> String {
    let mut bytes = [0u8; 8];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn request_airdrop(rpc: &RpcClient, pubkey: Pubkey, lamports: u64) -> Result<()> {
    let sig = rpc.request_airdrop(&pubkey, lamports)?;
    rpc.confirm_transaction(&sig)?;
    Ok(())
}

#[allow(deprecated)]
fn create_mint_and_fund(
    rpc: &RpcClient,
    depositor: &Keypair,
    amount: u64,
    decimals: u8,
) -> Result<Pubkey> {
    let mint = Keypair::new();
    let rent = rpc.get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)?;
    let create_ix = create_account(
        &depositor.pubkey(),
        &mint.pubkey(),
        rent,
        spl_token::state::Mint::LEN as u64,
        &spl_token::ID,
    );
    let init_ix = token_instruction::initialize_mint(
        &spl_token::ID,
        &mint.pubkey(),
        &depositor.pubkey(),
        None,
        decimals,
    )?;

    let ata = get_associated_token_address(&depositor.pubkey(), &mint.pubkey());
    let mut ixs = vec![create_ix, init_ix];
    if rpc.get_account(&ata).is_err() {
        ixs.push(create_associated_token_account(
            &depositor.pubkey(),
            &depositor.pubkey(),
            &mint.pubkey(),
            &spl_token::ID,
        ));
    }
    ixs.push(token_instruction::mint_to(
        &spl_token::ID,
        &mint.pubkey(),
        &ata,
        &depositor.pubkey(),
        &[],
        amount,
    )?);

    let blockhash = rpc.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &ixs,
        Some(&depositor.pubkey()),
        &[depositor, &mint],
        blockhash,
    );
    rpc.send_and_confirm_transaction(&tx)?;
    Ok(mint.pubkey())
}

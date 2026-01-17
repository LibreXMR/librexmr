use anyhow::{anyhow, Context, Result};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::claim::verify_hashlock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRecord {
    pub swap_id: String,
    pub hashlock: String,
    pub amount_piconero: u64,
    pub locked_at: u64,
    pub claimed_at: Option<u64>,
    pub secret: Option<String>,
    pub claim_txid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WalletState {
    swaps: Vec<SwapRecord>,
}

pub struct MockWallet {
    state_path: PathBuf,
}

impl MockWallet {
    pub fn new<P: AsRef<Path>>(state_dir: P) -> Result<Self> {
        let state_dir = state_dir.as_ref().to_path_buf();
        fs::create_dir_all(&state_dir)
            .with_context(|| format!("Failed to create directory: {:?}", state_dir))?;
        let state_path = state_dir.join("xmr_simulator_state.json");
        if !state_path.exists() {
            let empty = WalletState { swaps: vec![] };
            fs::write(&state_path, serde_json::to_string_pretty(&empty)?)?;
        }
        Ok(Self { state_path })
    }

    pub fn lock(
        &self,
        swap_id: &str,
        hashlock: &str,
        amount_piconero: u64,
        now_unix: u64,
    ) -> Result<SwapRecord> {
        let mut state = self.load_state()?;
        if state.swaps.iter().any(|swap| swap.swap_id == swap_id) {
            return Err(anyhow!("swap already exists: {swap_id}"));
        }
        let record = SwapRecord {
            swap_id: swap_id.to_string(),
            hashlock: hashlock.trim().to_lowercase(),
            amount_piconero,
            locked_at: now_unix,
            claimed_at: None,
            secret: None,
            claim_txid: None,
        };
        state.swaps.push(record.clone());
        self.save_state(&state)?;
        Ok(record)
    }

    pub fn claim(&self, swap_id: &str, secret_hex: &str, now_unix: u64) -> Result<SwapRecord> {
        let mut state = self.load_state()?;
        let updated = {
            let record = state
                .swaps
                .iter_mut()
                .find(|swap| swap.swap_id == swap_id)
                .ok_or_else(|| anyhow!("swap not found: {swap_id}"))?;

            if record.claimed_at.is_some() {
                return Err(anyhow!("swap already claimed: {swap_id}"));
            }
            if !verify_hashlock(secret_hex, &record.hashlock)? {
                return Err(anyhow!("hashlock mismatch: invalid secret"));
            }

            record.claimed_at = Some(now_unix);
            record.secret = Some(secret_hex.trim().to_lowercase());
            record.claim_txid = Some(random_txid());
            record.clone()
        };

        self.save_state(&state)?;
        Ok(updated)
    }

    fn load_state(&self) -> Result<WalletState> {
        let raw = fs::read_to_string(&self.state_path)
            .with_context(|| format!("Failed to read {:?}", self.state_path))?;
        let state: WalletState = serde_json::from_str(&raw)
            .with_context(|| format!("Failed to parse {:?}", self.state_path))?;
        Ok(state)
    }

    fn save_state(&self, state: &WalletState) -> Result<()> {
        fs::write(&self.state_path, serde_json::to_string_pretty(state)?)
            .with_context(|| format!("Failed to write {:?}", self.state_path))?;
        Ok(())
    }
}

fn random_txid() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

//! Swap state machine driver.
//!
//! Ported and adapted from:
//! https://github.com/omarespejel/monero-starknet-atomic-swap (Omar Espejel)

use anyhow::{anyhow, Result};
use async_trait::async_trait;

use crate::db::SwapDb;
use crate::state::SwapState;

#[async_trait]
pub trait SolanaClient: Send + Sync {
    async fn initialize(&self, lock_duration_secs: u64) -> Result<(String, String, i64, String)>;
    async fn verify_dleq(&self, lock_pda: &str) -> Result<String>;
    async fn unlock(&self, lock_pda: &str, vault: &str, secret: [u8; 32]) -> Result<String>;
    async fn refund(&self, lock_pda: &str, vault: &str) -> Result<String>;
    async fn get_block_timestamp(&self) -> Result<i64>;
}

pub async fn step<D, S>(
    state: &SwapState,
    db: &D,
    client: &S,
    secret: Option<[u8; 32]>,
) -> Result<Option<SwapState>>
where
    D: SwapDb,
    S: SolanaClient,
{
    if let Some(lock_until) = get_lock_until(state) {
        let now = client.get_block_timestamp().await?;
        if now >= lock_until && can_refund(state) {
            let new_state = handle_refund(state, client).await?;
            db.save(&new_state)?;
            return Ok(Some(new_state));
        }
    }

    let new_state = match state {
        SwapState::Created { swap_id, lock_duration_secs, .. } => {
            let (lock_pda, vault, lock_until, sig) = client.initialize(*lock_duration_secs).await?;
            SwapState::Initialized {
                swap_id: swap_id.clone(),
                lock_pda,
                vault,
                lock_until,
                token_mint: token_mint_from_state(state),
                amount: amount_from_state(state),
                initialize_tx: sig,
            }
        }
        SwapState::Initialized { swap_id, lock_pda, vault, lock_until, .. } => {
            let sig = client.verify_dleq(lock_pda).await?;
            SwapState::DleqVerified {
                swap_id: swap_id.clone(),
                lock_pda: lock_pda.clone(),
                vault: vault.clone(),
                lock_until: *lock_until,
                token_mint: token_mint_from_state(state),
                amount: amount_from_state(state),
                verify_tx: sig,
            }
        }
        SwapState::DleqVerified { swap_id, lock_pda, vault, .. } => {
            let secret = secret.ok_or_else(|| anyhow!("secret required to unlock"))?;
            let sig = client.unlock(lock_pda, vault, secret).await?;
            SwapState::Unlocked {
                swap_id: swap_id.clone(),
                unlock_tx: sig,
            }
        }
        SwapState::Unlocked { .. } | SwapState::Refunded { .. } => {
            return Ok(None);
        }
    };

    db.save(&new_state)?;
    Ok(Some(new_state))
}

fn get_lock_until(state: &SwapState) -> Option<i64> {
    match state {
        SwapState::Initialized { lock_until, .. }
        | SwapState::DleqVerified { lock_until, .. } => Some(*lock_until),
        _ => None,
    }
}

fn can_refund(state: &SwapState) -> bool {
    matches!(state, SwapState::Initialized { .. } | SwapState::DleqVerified { .. })
}

async fn handle_refund<S: SolanaClient>(state: &SwapState, client: &S) -> Result<SwapState> {
    let (swap_id, lock_pda, vault) = match state {
        SwapState::Initialized { swap_id, lock_pda, vault, .. }
        | SwapState::DleqVerified { swap_id, lock_pda, vault, .. } => {
            (swap_id.clone(), lock_pda.clone(), vault.clone())
        }
        _ => return Err(anyhow!("Cannot refund from state: {:?}", state)),
    };

    let refund_tx = client.refund(&lock_pda, &vault).await?;
    Ok(SwapState::Refunded {
        swap_id,
        reason: "Timeout exceeded".to_string(),
        refund_tx: Some(refund_tx),
    })
}

fn token_mint_from_state(state: &SwapState) -> String {
    match state {
        SwapState::Created { token_mint, .. }
        | SwapState::Initialized { token_mint, .. }
        | SwapState::DleqVerified { token_mint, .. } => token_mint.clone(),
        _ => String::new(),
    }
}

fn amount_from_state(state: &SwapState) -> u64 {
    match state {
        SwapState::Created { amount, .. }
        | SwapState::Initialized { amount, .. }
        | SwapState::DleqVerified { amount, .. } => *amount,
        _ => 0,
    }
}

//! Swap state machine.
//!
//! Ported and adapted from:
//! https://github.com/omarespejel/monero-starknet-atomic-swap (Omar Espejel)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SwapState {
    Created {
        swap_id: String,
        depositor: String,
        unlocker: String,
        token_mint: String,
        amount: u64,
        lock_duration_secs: u64,
        hashlock: [u8; 32],
        adaptor_point: [u8; 32],
        second_point: [u8; 32],
        y_point: [u8; 32],
        r1: [u8; 32],
        r2: [u8; 32],
        challenge: [u8; 32],
        response: [u8; 32],
    },
    Initialized {
        swap_id: String,
        lock_pda: String,
        vault: String,
        lock_until: i64,
        token_mint: String,
        amount: u64,
        initialize_tx: String,
    },
    DleqVerified {
        swap_id: String,
        lock_pda: String,
        vault: String,
        lock_until: i64,
        token_mint: String,
        amount: u64,
        verify_tx: String,
    },
    Unlocked {
        swap_id: String,
        unlock_tx: String,
    },
    Refunded {
        swap_id: String,
        reason: String,
        refund_tx: Option<String>,
    },
}

impl SwapState {
    pub fn swap_id(&self) -> &str {
        match self {
            Self::Created { swap_id, .. }
            | Self::Initialized { swap_id, .. }
            | Self::DleqVerified { swap_id, .. }
            | Self::Unlocked { swap_id, .. }
            | Self::Refunded { swap_id, .. } => swap_id,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Unlocked { .. } | Self::Refunded { .. })
    }
}

use monero_rpc::TransferHeight;
use tracing::{debug, warn};

use crate::rpc::XmrWallet;
use crate::types::Result;

#[derive(Debug, Clone, Copy)]
pub struct WatcherConfig {
    pub confirmations_required: u64,
    pub reorg_buffer: u64,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            confirmations_required: 10,
            reorg_buffer: 5,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WatcherState {
    last_seen_height: Option<u64>,
    lock_height: Option<u64>,
}

impl WatcherState {
    pub fn new() -> Self {
        Self {
            last_seen_height: None,
            lock_height: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatcherEvent {
    NoLockObserved,
    LockInPool { amount: u64 },
    AwaitingConfirmations {
        observed_height: u64,
        current_height: u64,
        remaining: u64,
    },
    Confirmed {
        observed_height: u64,
        confirmations: u64,
    },
    ReorgDetected {
        previous_height: u64,
        current_height: u64,
    },
}

pub struct LockWatcher {
    config: WatcherConfig,
    state: WatcherState,
}

impl LockWatcher {
    pub fn new(config: WatcherConfig) -> Self {
        Self {
            config,
            state: WatcherState::new(),
        }
    }

    pub fn observe_lock(&mut self, height: u64) {
        let should_update = self.state.lock_height.map_or(true, |current| height < current);
        if should_update {
            debug!(lock_height = height, "Observed lock height");
            self.state.lock_height = Some(height);
        }
    }

    pub fn update_height(&mut self, current_height: u64) -> Option<WatcherEvent> {
        if let Some(previous) = self.state.last_seen_height {
            if current_height + self.config.reorg_buffer < previous {
                warn!(
                    previous_height = previous,
                    current_height,
                    "Reorg detected; clearing observed lock height"
                );
                self.state.last_seen_height = Some(current_height);
                self.state.lock_height = None;
                return Some(WatcherEvent::ReorgDetected {
                    previous_height: previous,
                    current_height,
                });
            }
        }

        self.state.last_seen_height = Some(current_height);
        None
    }

    pub fn evaluate(&self, current_height: u64) -> WatcherEvent {
        match self.state.lock_height {
            None => WatcherEvent::NoLockObserved,
            Some(observed_height) => {
                let required = self.config.confirmations_required;
                let confirmations = current_height.saturating_sub(observed_height);
                if confirmations >= required {
                    WatcherEvent::Confirmed {
                        observed_height,
                        confirmations,
                    }
                } else {
                    WatcherEvent::AwaitingConfirmations {
                        observed_height,
                        current_height,
                        remaining: required - confirmations,
                    }
                }
            }
        }
    }

    pub fn state(&self) -> WatcherState {
        self.state
    }

    pub async fn poll_for_lock(
        &mut self,
        wallet: &XmrWallet,
        expected_amount: u64,
    ) -> Result<Option<WatcherEvent>> {
        let current_height = wallet.get_height().await?;
        if let Some(event) = self.update_height(current_height) {
            return Ok(Some(event));
        }

        wallet.refresh(self.state.last_seen_height).await?;
        let transfers = wallet.get_incoming_transfers().await?;
        for transfer in transfers {
            let amount = transfer.amount.as_pico();
            if amount < expected_amount {
                continue;
            }
            match transfer.height {
                TransferHeight::InPool => {
                    debug!(amount, "Observed lock transfer in pool");
                    return Ok(Some(WatcherEvent::LockInPool { amount }));
                }
                TransferHeight::Confirmed(height) => {
                    let observed_height = height.get();
                    self.observe_lock(observed_height);
                    return Ok(Some(self.evaluate(current_height)));
                }
            }
        }

        Ok(None)
    }
}

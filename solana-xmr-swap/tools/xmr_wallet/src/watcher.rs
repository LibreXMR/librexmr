use tracing::{debug, warn};

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
}

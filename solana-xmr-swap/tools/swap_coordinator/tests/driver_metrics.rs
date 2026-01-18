use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;

use swap_coordinator::driver::{step, SolanaClient};
use swap_coordinator::metrics::SwapMetrics;
use swap_coordinator::state::SwapState;
use swap_coordinator::db::SwapDb;

struct MemoryDb {
    states: Arc<Mutex<Vec<SwapState>>>,
}

impl MemoryDb {
    fn new() -> Self {
        Self {
            states: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl SwapDb for MemoryDb {
    fn save(&self, state: &SwapState) -> Result<()> {
        self.states.lock().unwrap().push(state.clone());
        Ok(())
    }

    fn load(&self, _swap_id: &str) -> Result<Option<SwapState>> {
        Ok(None)
    }
}

struct DummyClient;

#[async_trait::async_trait]
impl SolanaClient for DummyClient {
    async fn initialize(&self, _lock_duration_secs: u64) -> Result<(String, String, i64, String)> {
        Ok((
            "lock".to_string(),
            "vault".to_string(),
            9999,
            "init_sig".to_string(),
        ))
    }

    async fn verify_dleq(&self, _lock_pda: &str) -> Result<String> {
        Ok("verify_sig".to_string())
    }

    async fn unlock(&self, _lock_pda: &str, _vault: &str, _secret: [u8; 32]) -> Result<String> {
        Ok("unlock_sig".to_string())
    }

    async fn refund(&self, _lock_pda: &str, _vault: &str) -> Result<String> {
        Ok("refund_sig".to_string())
    }

    async fn get_block_timestamp(&self) -> Result<i64> {
        Ok(0)
    }
}

#[derive(Default)]
struct TestMetrics {
    transitions: Arc<Mutex<Vec<(String, String)>>>,
    latencies: Arc<Mutex<Vec<String>>>,
}

impl SwapMetrics for TestMetrics {
    fn record_transition(&self, from: &SwapState, to: &SwapState) {
        self.transitions.lock().unwrap().push((state_name(from), state_name(to)));
    }

    fn record_latency(&self, stage: &'static str, _elapsed: Duration) {
        self.latencies.lock().unwrap().push(stage.to_string());
    }
}

fn state_name(state: &SwapState) -> String {
    match state {
        SwapState::Created { .. } => "created",
        SwapState::Initialized { .. } => "initialized",
        SwapState::DleqVerified { .. } => "dleq_verified",
        SwapState::Unlocked { .. } => "unlocked",
        SwapState::Refunded { .. } => "refunded",
    }
    .to_string()
}

fn demo_state() -> SwapState {
    SwapState::Created {
        swap_id: "swap".to_string(),
        depositor: "dep".to_string(),
        unlocker: "unlock".to_string(),
        token_mint: "mint".to_string(),
        amount: 1,
        lock_duration_secs: 1,
        hashlock: [0u8; 32],
        adaptor_point: [1u8; 32],
        second_point: [2u8; 32],
        y_point: [3u8; 32],
        r1: [4u8; 32],
        r2: [5u8; 32],
        challenge: [6u8; 32],
        response: [7u8; 32],
    }
}

#[tokio::test]
async fn metrics_are_emitted_on_transition() {
    let db = MemoryDb::new();
    let client = DummyClient;
    let metrics = TestMetrics::default();
    let state = demo_state();

    let next = step(&state, &db, &client, &metrics, None).await.unwrap();
    assert!(next.is_some());

    let transitions = metrics.transitions.lock().unwrap();
    assert_eq!(transitions.len(), 1);
    assert_eq!(transitions[0].0, "created");
    assert_eq!(transitions[0].1, "initialized");

    let latencies = metrics.latencies.lock().unwrap();
    assert_eq!(latencies.as_slice(), &["initialize"]);
}

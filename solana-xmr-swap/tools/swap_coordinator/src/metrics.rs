use std::time::Duration;

use crate::state::SwapState;

pub trait SwapMetrics: Send + Sync {
    fn record_transition(&self, from: &SwapState, to: &SwapState);
    fn record_latency(&self, stage: &'static str, elapsed: Duration);
}

#[derive(Default, Debug, Clone)]
pub struct NoopMetrics;

impl SwapMetrics for NoopMetrics {
    fn record_transition(&self, _from: &SwapState, _to: &SwapState) {}
    fn record_latency(&self, _stage: &'static str, _elapsed: Duration) {}
}

#[derive(Default, Debug, Clone)]
#[allow(dead_code)]
pub struct StdoutMetrics;

impl SwapMetrics for StdoutMetrics {
    fn record_transition(&self, from: &SwapState, to: &SwapState) {
        println!("metrics.transition from={:?} to={:?}", from, to);
    }

    fn record_latency(&self, stage: &'static str, elapsed: Duration) {
        println!(
            "metrics.latency stage={} elapsed_ms={}",
            stage,
            elapsed.as_millis()
        );
    }
}

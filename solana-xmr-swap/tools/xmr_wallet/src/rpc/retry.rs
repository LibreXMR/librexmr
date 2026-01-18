use std::time::Duration;

use rand::Rng;
use tokio::time::{sleep, timeout};
use tracing::warn;

use crate::types::Result;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub timeout: Duration,
    pub max_retries: usize,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(6_000),
            max_retries: 3,
            base_delay: Duration::from_millis(250),
            max_delay: Duration::from_secs(5),
            jitter_ms: 250,
        }
    }
}

impl RetryConfig {
    pub fn from_env() -> Self {
        let default = Self::default();
        let timeout_ms = read_env_u64("XMR_WALLET_RPC_TIMEOUT_MS", default.timeout.as_millis() as u64);
        let max_retries = read_env_u64("XMR_WALLET_RPC_MAX_RETRIES", default.max_retries as u64);
        let base_delay_ms = read_env_u64("XMR_WALLET_RPC_BASE_DELAY_MS", default.base_delay.as_millis() as u64);
        let max_delay_ms = read_env_u64("XMR_WALLET_RPC_MAX_DELAY_MS", default.max_delay.as_millis() as u64);
        let jitter_ms = read_env_u64("XMR_WALLET_RPC_JITTER_MS", default.jitter_ms);
        Self {
            timeout: Duration::from_millis(timeout_ms),
            max_retries: max_retries as usize,
            base_delay: Duration::from_millis(base_delay_ms),
            max_delay: Duration::from_millis(max_delay_ms),
            jitter_ms,
        }
    }
}

fn read_env_u64(key: &str, fallback: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(fallback)
}

pub async fn retry_with_timeout<T, F, Fut>(
    label: &'static str,
    config: &RetryConfig,
    mut action: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let attempts = config.max_retries.saturating_add(1);
    for attempt in 0..attempts {
        let outcome = timeout(config.timeout, action()).await;
        match outcome {
            Ok(Ok(value)) => return Ok(value),
            Ok(Err(err)) => {
                if attempt + 1 >= attempts {
                    return Err(err);
                }
                warn!(attempt = attempt + 1, "RPC error on {label}; retrying");
            }
            Err(_) => {
                if attempt + 1 >= attempts {
                    return Err(anyhow::anyhow!("RPC timeout for {label}").into());
                }
                warn!(attempt = attempt + 1, "RPC timeout on {label}; retrying");
            }
        }

        let backoff = config
            .base_delay
            .saturating_mul(2u32.saturating_pow(attempt as u32));
        let capped = std::cmp::min(backoff, config.max_delay);
        let jitter = if config.jitter_ms == 0 {
            Duration::from_millis(0)
        } else {
            Duration::from_millis(rand::thread_rng().gen_range(0..=config.jitter_ms))
        };
        sleep(capped + jitter).await;
    }

    Err(anyhow::anyhow!("RPC retry exhausted for {label}").into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn retries_until_success() {
        let config = RetryConfig {
            timeout: Duration::from_millis(50),
            max_retries: 2,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(2),
            jitter_ms: 0,
        };
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let calls_clone = calls.clone();
        let result = retry_with_timeout("test", &config, move || {
            let calls = calls_clone.clone();
            async move {
                let current = calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                if current < 3 {
                    Err(anyhow::anyhow!("fail").into())
                } else {
                    Ok(42)
                }
            }
        })
        .await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn times_out_and_exhausts_retries() {
        let config = RetryConfig {
            timeout: Duration::from_millis(5),
            max_retries: 1,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(2),
            jitter_ms: 0,
        };
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let calls_clone = calls.clone();
        let result = retry_with_timeout("timeout", &config, move || {
            let calls = calls_clone.clone();
            async move {
                calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                sleep(Duration::from_millis(20)).await;
                Ok::<_, crate::types::XmrWalletError>(())
            }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    }
}

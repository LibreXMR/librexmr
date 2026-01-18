use std::env;

use xmr_wallet::{LockWatcher, WatcherConfig, XmrWallet};

fn env_or_skip(key: &str) -> Option<String> {
    match env::var(key) {
        Ok(value) if !value.trim().is_empty() => Some(value),
        _ => {
            eprintln!("Skipping test: set {} to run watcher RPC tests.", key);
            None
        }
    }
}

#[tokio::test]
#[ignore = "requires monero-wallet-rpc running locally"]
async fn watcher_poll_uses_rpc() {
    let url = match env_or_skip("MONERO_WALLET_RPC_URL") {
        Some(value) => value,
        None => return,
    };

    let wallet = XmrWallet::connect(&url).await.expect("connect wallet RPC");
    let mut watcher = LockWatcher::new(WatcherConfig::default());
    let result = watcher.poll_for_lock(&wallet, 1).await;
    assert!(result.is_ok());
}

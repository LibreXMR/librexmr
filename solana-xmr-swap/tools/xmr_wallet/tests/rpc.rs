use std::env;

use xmr_wallet::XmrWallet;

fn env_or_skip(key: &str) -> Option<String> {
    match env::var(key) {
        Ok(value) if !value.trim().is_empty() => Some(value),
        _ => {
            eprintln!("Skipping test: set {} to run RPC tests.", key);
            None
        }
    }
}

#[tokio::test]
#[ignore = "requires monero-wallet-rpc running locally"]
async fn rpc_create_open_address_balance() {
    let url = match env_or_skip("MONERO_WALLET_RPC_URL") {
        Some(value) => value,
        None => return,
    };
    let wallet_name = env::var("MONERO_WALLET_NAME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "audit_test_wallet".to_string());
    let wallet_password = env::var("MONERO_WALLET_PASSWORD")
        .ok()
        .filter(|value| !value.trim().is_empty());
    let wallet_language = env::var("MONERO_WALLET_LANGUAGE")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "English".to_string());

    let wallet = XmrWallet::connect(&url).await.expect("connect wallet RPC");

    if wallet
        .create_wallet(&wallet_name, wallet_password.as_deref(), &wallet_language)
        .await
        .is_err()
    {
        wallet
            .open_wallet(&wallet_name, wallet_password.as_deref())
            .await
            .expect("open existing wallet");
    }

    let address = wallet
        .get_primary_address()
        .await
        .expect("get primary address");
    assert!(!address.is_empty());

    let _balance = wallet.get_balance().await.expect("get balance");
}

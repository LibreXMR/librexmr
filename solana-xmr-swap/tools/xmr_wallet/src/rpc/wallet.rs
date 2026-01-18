use std::collections::HashMap;
use std::str::FromStr;

use monero_rpc::{
    monero::{Address, PrivateKey},
    GetTransfersCategory, GetTransfersSelector, GotTransfer, RpcClientBuilder, SweepAllArgs,
    TransferPriority, WalletClient,
};
use tracing::debug;

use anyhow::anyhow;

use crate::rpc::{retry_with_timeout, RetryConfig};
use crate::types::Result;

pub struct XmrWallet {
    client: WalletClient,
    retry: RetryConfig,
}

impl XmrWallet {
    pub async fn connect(url: &str) -> Result<Self> {
        let retry = RetryConfig::from_env();
        Self::connect_with_config(url, retry).await
    }

    pub async fn connect_with_config(url: &str, retry: RetryConfig) -> Result<Self> {
        debug!(rpc_url = url, "Connecting to Monero wallet RPC");
        let rpc_client = RpcClientBuilder::new().build(url)?;
        let client = rpc_client.wallet();
        Ok(Self { client, retry })
    }

    async fn with_retry<T, F, Fut>(&self, label: &'static str, action: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        retry_with_timeout(label, &self.retry, action).await
    }

    pub async fn create_wallet(
        &self,
        filename: &str,
        password: Option<&str>,
        language: &str,
    ) -> Result<()> {
        debug!(wallet = filename, "Creating Monero wallet");
        let filename = filename.to_string();
        let password = password.map(|value| value.to_string());
        let language = language.to_string();
        self.with_retry("create_wallet", || async {
            self.client
                .create_wallet(filename.clone(), password.clone(), language.clone())
                .await?;
            Ok(())
        })
        .await
    }

    pub async fn open_wallet(&self, filename: &str, password: Option<&str>) -> Result<()> {
        debug!(wallet = filename, "Opening Monero wallet");
        let filename = filename.to_string();
        let password = password.map(|value| value.to_string());
        self.with_retry("open_wallet", || async {
            self.client
                .open_wallet(filename.clone(), password.clone())
                .await?;
            Ok(())
        })
        .await
    }

    pub async fn close_wallet(&self) -> Result<()> {
        debug!("Closing Monero wallet");
        self.with_retry("close_wallet", || async {
            self.client.close_wallet().await?;
            Ok(())
        })
        .await
    }

    pub async fn get_balance(&self) -> Result<u64> {
        debug!("Fetching Monero wallet balance");
        self.with_retry("get_balance", || async {
            let balance = self.client.get_balance(0, None).await?;
            Ok(balance.balance.as_pico())
        })
        .await
    }

    pub async fn get_primary_address(&self) -> Result<String> {
        debug!("Fetching Monero primary address");
        self.with_retry("get_primary_address", || async {
            let addresses = self.client.get_address(0, None).await?;
            Ok(addresses.address.to_string())
        })
        .await
    }

    pub async fn refresh(&self, start_height: Option<u64>) -> Result<()> {
        debug!(start_height, "Refreshing Monero wallet");
        self.with_retry("refresh", || async {
            self.client.refresh(start_height).await?;
            Ok(())
        })
        .await
    }

    pub async fn get_height(&self) -> Result<u64> {
        debug!("Fetching Monero wallet height");
        self.with_retry("get_height", || async {
            let height = self.client.get_height().await?;
            Ok(height.get())
        })
        .await
    }

    pub async fn get_incoming_transfers(&self) -> Result<Vec<GotTransfer>> {
        self.with_retry("get_incoming_transfers", || async {
            let mut selector = GetTransfersSelector::default();
            let mut category_selector = HashMap::new();
            category_selector.insert(GetTransfersCategory::In, true);
            selector.category_selector = category_selector;
            let transfers = self.client.get_transfers(selector).await?;
            let mut results = Vec::new();
            for values in transfers.values() {
                results.extend(values.clone());
            }
            Ok(results)
        })
        .await
    }

    pub async fn import_spend_key(
        &self,
        spend_key_bytes: [u8; 32],
        view_key_bytes: [u8; 32],
        address: &str,
        filename: &str,
        restore_height: Option<u64>,
    ) -> Result<()> {
        debug!(wallet = filename, "Importing spend/view keys into wallet");
        let address_str = address.to_string();
        PrivateKey::from_slice(&spend_key_bytes)
            .map_err(|err| anyhow!("invalid spend key: {err}"))?;
        PrivateKey::from_slice(&view_key_bytes).map_err(|err| anyhow!("invalid view key: {err}"))?;
        Address::from_str(&address_str).map_err(|err| anyhow!("invalid address: {err}"))?;
        self.with_retry("generate_from_keys", || async {
            let spend_key = PrivateKey::from_slice(&spend_key_bytes)
                .map_err(|err| anyhow!("invalid spend key: {err}"))?;
            let view_key = PrivateKey::from_slice(&view_key_bytes)
                .map_err(|err| anyhow!("invalid view key: {err}"))?;
            let address =
                Address::from_str(&address_str).map_err(|err| anyhow!("invalid address: {err}"))?;
            let args = monero_rpc::GenerateFromKeysArgs {
                restore_height,
                filename: filename.to_string(),
                address,
                spendkey: Some(spend_key),
                viewkey: view_key,
                password: String::new(),
                autosave_current: Some(true),
            };
            self.client.generate_from_keys(args).await?;
            Ok(())
        })
        .await
    }

    pub async fn sweep_all(&self, destination: &str) -> Result<String> {
        debug!("Sweeping wallet balance");
        Address::from_str(destination).map_err(|err| anyhow!("invalid destination address: {err}"))?;
        let destination = destination.to_string();
        self.with_retry("sweep_all", || async {
            let address = Address::from_str(&destination)
                .map_err(|err| anyhow!("invalid destination address: {err}"))?;
            let args = SweepAllArgs {
                address,
                account_index: 0,
                subaddr_indices: None,
                priority: TransferPriority::Default,
                mixin: 15,
                ring_size: 16,
                unlock_time: 0,
                get_tx_keys: Some(true),
                below_amount: None,
                do_not_relay: None,
                get_tx_hex: None,
                get_tx_metadata: None,
            };
            let result = self.client.sweep_all(args).await?;
            let first = result
                .tx_hash_list
                .first()
                .map(|hash| hash.to_string())
                .unwrap_or_default();
            Ok(first)
        })
        .await
    }
}

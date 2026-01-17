use monero_rpc::{RpcClientBuilder, WalletClient};
use tracing::debug;

use crate::types::Result;

pub struct XmrWallet {
    client: WalletClient,
}

impl XmrWallet {
    pub async fn connect(url: &str) -> Result<Self> {
        debug!(rpc_url = url, "Connecting to Monero wallet RPC");
        let rpc_client = RpcClientBuilder::new().build(url)?;
        let client = rpc_client.wallet();
        Ok(Self { client })
    }

    pub async fn create_wallet(
        &self,
        filename: &str,
        password: Option<&str>,
        language: &str,
    ) -> Result<()> {
        debug!(wallet = filename, "Creating Monero wallet");
        self.client
            .create_wallet(
                filename.to_string(),
                password.map(|value| value.to_string()),
                language.to_string(),
            )
            .await?;
        Ok(())
    }

    pub async fn open_wallet(&self, filename: &str, password: Option<&str>) -> Result<()> {
        debug!(wallet = filename, "Opening Monero wallet");
        self.client
            .open_wallet(
                filename.to_string(),
                password.map(|value| value.to_string()),
            )
            .await?;
        Ok(())
    }

    pub async fn close_wallet(&self) -> Result<()> {
        debug!("Closing Monero wallet");
        self.client.close_wallet().await?;
        Ok(())
    }

    pub async fn get_balance(&self) -> Result<u64> {
        debug!("Fetching Monero wallet balance");
        let balance = self.client.get_balance(0, None).await?;
        Ok(balance.balance.as_pico())
    }

    pub async fn get_primary_address(&self) -> Result<String> {
        debug!("Fetching Monero primary address");
        let addresses = self.client.get_address(0, None).await?;
        Ok(addresses.address.to_string())
    }

    pub async fn refresh(&self, start_height: Option<u64>) -> Result<()> {
        debug!(start_height, "Refreshing Monero wallet");
        self.client.refresh(start_height).await?;
        Ok(())
    }
}

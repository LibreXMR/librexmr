use anyhow::{anyhow, Result};
use curve25519_dalek::scalar::Scalar;
use monero::Network;

use xmr_wallet::{execute_claim, ClaimParams, XmrWallet};

pub struct XmrClaimRequest {
    pub alice_partial: [u8; 32],
    pub bob_partial: [u8; 32],
    pub revealed_secret: [u8; 32],
    pub destination_address: String,
    pub wallet_filename: String,
    pub network: Network,
    pub restore_height: Option<u64>,
}

pub async fn claim_xmr(wallet: &XmrWallet, req: &XmrClaimRequest) -> Result<String> {
    let params = ClaimParams {
        alice_partial: Scalar::from_bytes_mod_order(req.alice_partial),
        bob_partial: Scalar::from_bytes_mod_order(req.bob_partial),
        revealed_secret: req.revealed_secret,
        destination_address: req.destination_address.clone(),
        network: req.network,
        wallet_filename: req.wallet_filename.clone(),
        restore_height: req.restore_height,
        expected_hashlock: None,
        claim_guard_path: None,
    };
    execute_claim(wallet, &params)
        .await
        .map_err(|err| anyhow!(err))
}

pub fn parse_network(value: &str) -> Result<Network> {
    match value.to_lowercase().as_str() {
        "mainnet" => Ok(Network::Mainnet),
        "testnet" => Ok(Network::Testnet),
        "stagenet" => Ok(Network::Stagenet),
        _ => Err(anyhow!("invalid network (expected mainnet/testnet/stagenet)")),
    }
}

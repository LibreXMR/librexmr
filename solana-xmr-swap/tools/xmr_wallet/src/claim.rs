use anyhow::anyhow;
use curve25519_dalek::scalar::Scalar;
use monero::{Address, Network, PrivateKey, PublicKey};
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::debug;
use zeroize::Zeroizing;

use crate::crypto::{derive_spend_key, derive_view_key, to_monero_private_key, SecretKey};
use crate::rpc::XmrWallet;
use crate::types::Result;

#[derive(Debug, Clone)]
pub struct ClaimParams {
    pub alice_partial: Scalar,
    pub bob_partial: Scalar,
    pub revealed_secret: Zeroizing<[u8; 32]>,
    pub destination_address: String,
    pub network: Network,
    pub wallet_filename: String,
    pub restore_height: Option<u64>,
    pub expected_hashlock: Option<[u8; 32]>,
    pub claim_guard_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ClaimKeys {
    pub spend_key: PrivateKey,
    pub view_key: PrivateKey,
    pub address: Address,
}

pub struct ClaimGuard {
    path: PathBuf,
    committed: bool,
}

impl ClaimGuard {
    pub fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for ClaimGuard {
    fn drop(&mut self) {
        if !self.committed {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

pub fn prepare_claim_guard(path: &Path) -> Result<ClaimGuard> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(anyhow::Error::from)?;
    Ok(ClaimGuard {
        path: path.to_path_buf(),
        committed: false,
    })
}

pub fn validate_destination_address(value: &str) -> Result<Address> {
    Ok(Address::from_str(value)?)
}

pub fn validate_hashlock(secret: &[u8; 32], expected: &[u8; 32]) -> Result<()> {
    let computed: [u8; 32] = Sha256::digest(secret).into();
    if &computed != expected {
        return Err(anyhow!("secret/hashlock mismatch").into());
    }
    Ok(())
}

fn revealed_secret_scalar(params: &ClaimParams) -> Scalar {
    Scalar::from_bytes_mod_order(*params.revealed_secret)
}

pub fn derive_claim_keys(params: &ClaimParams) -> Result<ClaimKeys> {
    let spend_scalar = derive_spend_key(
        &params.alice_partial,
        &params.bob_partial,
        &revealed_secret_scalar(params),
    );
    let spend_secret = SecretKey::from_scalar(spend_scalar);
    let view_secret = derive_view_key(&spend_secret)?;

    let spend_key = to_monero_private_key(&spend_secret)?;
    let view_key = to_monero_private_key(&view_secret)?;

    let public_spend = PublicKey::from_private_key(&spend_key);
    let public_view = PublicKey::from_private_key(&view_key);
    let address = Address::standard(params.network, public_spend, public_view);

    Ok(ClaimKeys {
        spend_key,
        view_key,
        address,
    })
}

pub async fn execute_claim(wallet: &XmrWallet, params: &ClaimParams) -> Result<String> {
    debug!("Executing Monero claim flow");
    validate_destination_address(&params.destination_address)?;
    if let Some(expected) = params.expected_hashlock.as_ref() {
        validate_hashlock(&*params.revealed_secret, expected)?;
    }
    let _guard = if let Some(path) = params.claim_guard_path.as_ref() {
        Some(prepare_claim_guard(path)?)
    } else {
        None
    };
    let keys = derive_claim_keys(params)?;
    wallet
        .import_spend_key(
            keys.spend_key.to_bytes(),
            keys.view_key.to_bytes(),
            &keys.address.to_string(),
            &params.wallet_filename,
            params.restore_height,
        )
        .await?;
    let tx_hash = wallet.sweep_all(&params.destination_address).await?;
    if let Some(guard) = _guard {
        guard.commit();
    }
    Ok(tx_hash)
}

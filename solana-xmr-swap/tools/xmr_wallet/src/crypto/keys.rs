use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};
use curve25519_dalek::scalar::Scalar;
use monero::cryptonote::hash::Hash;
use monero::util::key::PrivateKey as MoneroPrivateKey;
use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::Zeroize;

use crate::types::CryptoError;

#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct SecretKey([u8; 32]);

impl SecretKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn from_scalar(scalar: Scalar) -> Self {
        Self(scalar.to_bytes())
    }

    pub fn random() -> Self {
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    pub fn as_scalar(&self) -> Scalar {
        Scalar::from_bytes_mod_order(self.0)
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }
}

pub fn to_monero_private_key(secret: &SecretKey) -> Result<MoneroPrivateKey, CryptoError> {
    MoneroPrivateKey::from_slice(&secret.to_bytes()).map_err(CryptoError::from)
}

pub fn derive_view_key(spend_key: &SecretKey) -> Result<SecretKey, CryptoError> {
    let view = Hash::hash_to_scalar(spend_key.to_bytes());
    Ok(SecretKey::from_bytes(view.to_bytes()))
}

#[derive(Clone, Debug)]
pub struct PublicKey(EdwardsPoint);

impl PublicKey {
    pub fn from_compressed(bytes: [u8; 32]) -> Result<Self, CryptoError> {
        let point = CompressedEdwardsY(bytes)
            .decompress()
            .ok_or(CryptoError::InvalidPoint)?;

        if point.is_small_order() {
            return Err(CryptoError::SmallOrderPoint);
        }

        Ok(Self(point))
    }

    pub fn point(&self) -> &EdwardsPoint {
        &self.0
    }

    pub fn to_compressed(&self) -> [u8; 32] {
        self.0.compress().to_bytes()
    }
}

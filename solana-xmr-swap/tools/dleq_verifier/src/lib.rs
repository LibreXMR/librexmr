use blake2::{Blake2s256, Digest};
use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};
use curve25519_dalek::scalar::Scalar;
use serde::Deserialize;

const DLEQ_DOMAIN_TAG: &[u8; 4] = b"DLEQ";

#[derive(Debug, Deserialize)]
pub struct DleqVector {
    pub adaptor_point_compressed: String,
    pub challenge: String,
    pub dleq_second_point_compressed: String,
    pub hashlock: String,
    pub r1_compressed: String,
    pub r2_compressed: String,
    pub response: String,
    pub y_compressed: String,
}

#[derive(Debug)]
pub struct DleqInputs {
    pub adaptor_point: [u8; 32],
    pub second_point: [u8; 32],
    pub y_point: [u8; 32],
    pub r1: [u8; 32],
    pub r2: [u8; 32],
    pub challenge: [u8; 32],
    pub response: [u8; 32],
    pub hashlock: [u8; 32],
}

#[derive(Debug)]
pub struct DleqReport {
    pub computed_challenge: [u8; 32],
    pub challenge_matches: bool,
    pub lhs_r1_matches: bool,
    pub lhs_r2_matches: bool,
}

#[derive(Debug)]
pub enum VerifyError {
    InvalidHex { field: &'static str, reason: String },
    InvalidLength { field: &'static str, expected: usize, got: usize },
    InvalidPoint { field: &'static str },
    SmallOrderPoint { field: &'static str },
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifyError::InvalidHex { field, reason } => {
                write!(f, "invalid hex for {field}: {reason}")
            }
            VerifyError::InvalidLength { field, expected, got } => {
                write!(f, "invalid length for {field}: expected {expected}, got {got}")
            }
            VerifyError::InvalidPoint { field } => write!(f, "invalid point encoding: {field}"),
            VerifyError::SmallOrderPoint { field } => write!(f, "small-order point: {field}"),
        }
    }
}

impl std::error::Error for VerifyError {}

impl DleqVector {
    pub fn decode(&self) -> Result<DleqInputs, VerifyError> {
        Ok(DleqInputs {
            adaptor_point: decode_hex_32(&self.adaptor_point_compressed, "adaptor_point_compressed")?,
            second_point: decode_hex_32(
                &self.dleq_second_point_compressed,
                "dleq_second_point_compressed",
            )?,
            y_point: decode_hex_32(&self.y_compressed, "y_compressed")?,
            r1: decode_hex_32(&self.r1_compressed, "r1_compressed")?,
            r2: decode_hex_32(&self.r2_compressed, "r2_compressed")?,
            challenge: decode_hex_32(&self.challenge, "challenge")?,
            response: decode_hex_32(&self.response, "response")?,
            hashlock: decode_hex_32(&self.hashlock, "hashlock")?,
        })
    }
}

pub fn compute_challenge_bytes(
    y_point: &[u8; 32],
    adaptor_point: &[u8; 32],
    second_point: &[u8; 32],
    r1: &[u8; 32],
    r2: &[u8; 32],
    hashlock: &[u8; 32],
) -> [u8; 32] {
    let mut hasher = Blake2s256::new();
    hasher.update(DLEQ_DOMAIN_TAG);
    hasher.update(ED25519_BASEPOINT_POINT.compress().to_bytes());
    hasher.update(y_point);
    hasher.update(adaptor_point);
    hasher.update(second_point);
    hasher.update(r1);
    hasher.update(r2);
    hasher.update(hashlock);
    let hash: [u8; 32] = hasher.finalize().into();
    Scalar::from_bytes_mod_order(hash).to_bytes()
}

pub fn verify_dleq(inputs: &DleqInputs) -> Result<(bool, DleqReport), VerifyError> {
    let computed_challenge = compute_challenge_bytes(
        &inputs.y_point,
        &inputs.adaptor_point,
        &inputs.second_point,
        &inputs.r1,
        &inputs.r2,
        &inputs.hashlock,
    );

    let challenge_matches = computed_challenge == inputs.challenge;
    if !challenge_matches {
        return Ok((
            false,
            DleqReport {
                computed_challenge,
                challenge_matches,
                lhs_r1_matches: false,
                lhs_r2_matches: false,
            },
        ));
    }

    let t_point = decompress_point(&inputs.adaptor_point, "adaptor_point")?;
    let u_point = decompress_point(&inputs.second_point, "second_point")?;
    let y_point = decompress_point(&inputs.y_point, "y_point")?;
    let r1_point = decompress_point(&inputs.r1, "r1")?;
    let r2_point = decompress_point(&inputs.r2, "r2")?;

    ensure_not_small_order(&t_point, "adaptor_point")?;
    ensure_not_small_order(&u_point, "second_point")?;
    ensure_not_small_order(&y_point, "y_point")?;
    ensure_not_small_order(&r1_point, "r1")?;
    ensure_not_small_order(&r2_point, "r2")?;

    let c = Scalar::from_bytes_mod_order(inputs.challenge);
    let s = Scalar::from_bytes_mod_order(inputs.response);

    let s_g = &ED25519_BASEPOINT_POINT * &s;
    let s_y = &y_point * &s;
    let c_t = &t_point * &c;
    let c_u = &u_point * &c;

    let r1_check = r1_point + c_t;
    let r2_check = r2_point + c_u;

    let lhs_r1_matches = s_g == r1_check;
    let lhs_r2_matches = s_y == r2_check;

    Ok((
        lhs_r1_matches && lhs_r2_matches,
        DleqReport {
            computed_challenge,
            challenge_matches,
            lhs_r1_matches,
            lhs_r2_matches,
        },
    ))
}

fn decompress_point(bytes: &[u8; 32], field: &'static str) -> Result<EdwardsPoint, VerifyError> {
    CompressedEdwardsY(*bytes)
        .decompress()
        .ok_or(VerifyError::InvalidPoint { field })
}

fn ensure_not_small_order(point: &EdwardsPoint, field: &'static str) -> Result<(), VerifyError> {
    if point.is_small_order() {
        return Err(VerifyError::SmallOrderPoint { field });
    }
    Ok(())
}

fn decode_hex_32(value: &str, field: &'static str) -> Result<[u8; 32], VerifyError> {
    let mut out = [0u8; 32];
    let bytes = decode_hex(value, field)?;
    if bytes.len() != 32 {
        return Err(VerifyError::InvalidLength {
            field,
            expected: 32,
            got: bytes.len(),
        });
    }
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn decode_hex(value: &str, field: &'static str) -> Result<Vec<u8>, VerifyError> {
    if value.len() % 2 != 0 {
        return Err(VerifyError::InvalidHex {
            field,
            reason: "odd length".to_string(),
        });
    }
    let mut out = Vec::with_capacity(value.len() / 2);
    let mut iter = value.as_bytes().chunks_exact(2);
    while let Some(pair) = iter.next() {
        let hi = from_hex_digit(pair[0], field)?;
        let lo = from_hex_digit(pair[1], field)?;
        out.push((hi << 4) | lo);
    }
    Ok(out)
}

fn from_hex_digit(byte: u8, field: &'static str) -> Result<u8, VerifyError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(VerifyError::InvalidHex {
            field,
            reason: format!("invalid hex digit: 0x{byte:02x}"),
        }),
    }
}

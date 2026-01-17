 use anchor_lang::prelude::*;
 use blake2::{Blake2s256, Digest};
 use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};
use curve25519_dalek::scalar::Scalar;

use crate::errors::ErrorCode;
 
 const DLEQ_DOMAIN_TAG: &[u8; 4] = b"DLEQ";
 
struct DleqPoints {
    t_point: EdwardsPoint,
    u_point: EdwardsPoint,
    y_point: EdwardsPoint,
    r1_point: EdwardsPoint,
    r2_point: EdwardsPoint,
}

fn load_points_and_check_challenge(
    adaptor_point: &[u8; 32],
    second_point: &[u8; 32],
    y_point: &[u8; 32],
    r1: &[u8; 32],
    r2: &[u8; 32],
    challenge: &[u8; 32],
    hashlock: &[u8; 32],
) -> Result<Option<DleqPoints>> {
    let t_point = decompress_point(adaptor_point)?;
    let u_point = decompress_point(second_point)?;
    let y_point = decompress_point(y_point)?;
    let r1_point = decompress_point(r1)?;
    let r2_point = decompress_point(r2)?;

    if t_point.is_small_order()
        || u_point.is_small_order()
        || y_point.is_small_order()
        || r1_point.is_small_order()
        || r2_point.is_small_order()
    {
        return Err(error!(ErrorCode::InvalidPointEncoding));
    }

    let computed_challenge = compute_challenge_bytes(
        &ED25519_BASEPOINT_POINT,
        &y_point,
        &t_point,
        &u_point,
        &r1_point,
        &r2_point,
        hashlock,
    );

    if &computed_challenge != challenge {
        return Ok(None);
    }

    Ok(Some(DleqPoints {
        t_point,
        u_point,
        y_point,
        r1_point,
        r2_point,
    }))
}

pub fn verify_dleq_proof_bytes(
     adaptor_point: &[u8; 32],
     second_point: &[u8; 32],
     y_point: &[u8; 32],
     r1: &[u8; 32],
     r2: &[u8; 32],
     challenge: &[u8; 32],
     response: &[u8; 32],
     hashlock: &[u8; 32],
 ) -> Result<bool> {
    let Some(points) = load_points_and_check_challenge(
        adaptor_point,
        second_point,
        y_point,
        r1,
        r2,
        challenge,
        hashlock,
    )?
    else {
        return Ok(false);
    };

    let c = Scalar::from_bytes_mod_order(*challenge);
    let s = Scalar::from_bytes_mod_order(*response);
    let neg_c = -c;

    let lhs_r1 =
        EdwardsPoint::vartime_double_scalar_mul_basepoint(&neg_c, &points.t_point, &s);
    if lhs_r1 != points.r1_point {
        return Ok(false);
    }

    let lhs_r2 =
        EdwardsPoint::vartime_double_scalar_mul(&s, &points.y_point, &neg_c, &points.u_point);

    Ok(lhs_r2 == points.r2_point)
 }

pub fn verify_dleq_proof_part1_bytes(
    adaptor_point: &[u8; 32],
    second_point: &[u8; 32],
    y_point: &[u8; 32],
    r1: &[u8; 32],
    r2: &[u8; 32],
    challenge: &[u8; 32],
    response: &[u8; 32],
    hashlock: &[u8; 32],
) -> Result<bool> {
    let Some(points) = load_points_and_check_challenge(
        adaptor_point,
        second_point,
        y_point,
        r1,
        r2,
        challenge,
        hashlock,
    )?
    else {
        return Ok(false);
    };

    let c = Scalar::from_bytes_mod_order(*challenge);
    let s = Scalar::from_bytes_mod_order(*response);
    let neg_c = -c;

    let lhs_r1 =
        EdwardsPoint::vartime_double_scalar_mul_basepoint(&neg_c, &points.t_point, &s);

    Ok(lhs_r1 == points.r1_point)
}

pub fn verify_dleq_proof_part2_bytes(
    adaptor_point: &[u8; 32],
    second_point: &[u8; 32],
    y_point: &[u8; 32],
    r1: &[u8; 32],
    r2: &[u8; 32],
    challenge: &[u8; 32],
    response: &[u8; 32],
    hashlock: &[u8; 32],
) -> Result<bool> {
    let Some(points) = load_points_and_check_challenge(
        adaptor_point,
        second_point,
        y_point,
        r1,
        r2,
        challenge,
        hashlock,
    )?
    else {
        return Ok(false);
    };

    let c = Scalar::from_bytes_mod_order(*challenge);
    let s = Scalar::from_bytes_mod_order(*response);
    let neg_c = -c;

    let lhs_r2 =
        EdwardsPoint::vartime_double_scalar_mul(&s, &points.y_point, &neg_c, &points.u_point);

    Ok(lhs_r2 == points.r2_point)
}

pub fn validate_dleq_transcript_bytes(
    adaptor_point: &[u8; 32],
    second_point: &[u8; 32],
    y_point: &[u8; 32],
    r1: &[u8; 32],
    r2: &[u8; 32],
    challenge: &[u8; 32],
    hashlock: &[u8; 32],
) -> Result<bool> {
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
    let computed = Scalar::from_bytes_mod_order(hash).to_bytes();

    Ok(&computed == challenge)
}
 
 pub fn compute_challenge_bytes(
     g_point: &EdwardsPoint,
     y_point: &EdwardsPoint,
     t_point: &EdwardsPoint,
     u_point: &EdwardsPoint,
     r1_point: &EdwardsPoint,
     r2_point: &EdwardsPoint,
     hashlock: &[u8; 32],
 ) -> [u8; 32] {
     let mut hasher = Blake2s256::new();
     hasher.update(DLEQ_DOMAIN_TAG);
     hasher.update(g_point.compress().to_bytes());
     hasher.update(y_point.compress().to_bytes());
     hasher.update(t_point.compress().to_bytes());
     hasher.update(u_point.compress().to_bytes());
     hasher.update(r1_point.compress().to_bytes());
     hasher.update(r2_point.compress().to_bytes());
     hasher.update(hashlock);
    let hash: [u8; 32] = hasher.finalize().into();
    Scalar::from_bytes_mod_order(hash).to_bytes()
 }
 
 fn decompress_point(bytes: &[u8; 32]) -> Result<EdwardsPoint> {
     CompressedEdwardsY(*bytes)
         .decompress()
         .ok_or_else(|| error!(ErrorCode::InvalidPointEncoding))
 }
 
 #[cfg(test)]
 mod tests {
     use super::*;
    use curve25519_dalek::traits::Identity;
    use rand::rngs::OsRng;
    use sha2::Sha256;
 
     #[test]
     fn dleq_verifies_valid_proof() {
         let mut rng = OsRng;
 
         let t = Scalar::random(&mut rng);
         let k = Scalar::random(&mut rng);
         let y = Scalar::random(&mut rng);
 
         let y_point = &ED25519_BASEPOINT_POINT * &y;
         let t_point = &ED25519_BASEPOINT_POINT * &t;
         let u_point = &y_point * &t;
 
         let r1_point = &ED25519_BASEPOINT_POINT * &k;
         let r2_point = &y_point * &k;
 
         let hashlock: [u8; 32] = Sha256::digest(t.to_bytes()).into();
 
         let challenge = compute_challenge_bytes(
             &ED25519_BASEPOINT_POINT,
             &y_point,
             &t_point,
             &u_point,
             &r1_point,
             &r2_point,
             &hashlock,
         );
 
         let c = Scalar::from_bytes_mod_order(challenge);
         let s = k + c * t;
 
         let response = s.to_bytes();
 
         let ok = verify_dleq_proof_bytes(
             &t_point.compress().to_bytes(),
             &u_point.compress().to_bytes(),
             &y_point.compress().to_bytes(),
             &r1_point.compress().to_bytes(),
             &r2_point.compress().to_bytes(),
             &challenge,
             &response,
             &hashlock,
         )
         .unwrap();
 
         assert!(ok);
     }
 
     #[test]
     fn dleq_rejects_wrong_hashlock() {
         let mut rng = OsRng;
 
         let t = Scalar::random(&mut rng);
         let k = Scalar::random(&mut rng);
         let y = Scalar::random(&mut rng);
 
         let y_point = &ED25519_BASEPOINT_POINT * &y;
         let t_point = &ED25519_BASEPOINT_POINT * &t;
         let u_point = &y_point * &t;
 
         let r1_point = &ED25519_BASEPOINT_POINT * &k;
         let r2_point = &y_point * &k;
 
         let hashlock: [u8; 32] = Sha256::digest(t.to_bytes()).into();
         let mut bad_hashlock = hashlock;
         bad_hashlock[0] ^= 0x11;
 
         let challenge = compute_challenge_bytes(
             &ED25519_BASEPOINT_POINT,
             &y_point,
             &t_point,
             &u_point,
             &r1_point,
             &r2_point,
             &hashlock,
         );
 
         let c = Scalar::from_bytes_mod_order(challenge);
         let s = k + c * t;
         let response = s.to_bytes();
 
         let ok = verify_dleq_proof_bytes(
             &t_point.compress().to_bytes(),
             &u_point.compress().to_bytes(),
             &y_point.compress().to_bytes(),
             &r1_point.compress().to_bytes(),
             &r2_point.compress().to_bytes(),
             &challenge,
             &response,
             &bad_hashlock,
         )
         .unwrap();
 
         assert!(!ok);
     }

    #[test]
    fn dleq_rejects_tampered_response() {
        let mut rng = OsRng;

        let t = Scalar::random(&mut rng);
        let k = Scalar::random(&mut rng);
        let y = Scalar::random(&mut rng);

        let y_point = &ED25519_BASEPOINT_POINT * &y;
        let t_point = &ED25519_BASEPOINT_POINT * &t;
        let u_point = &y_point * &t;

        let r1_point = &ED25519_BASEPOINT_POINT * &k;
        let r2_point = &y_point * &k;

        let hashlock: [u8; 32] = Sha256::digest(t.to_bytes()).into();

        let challenge = compute_challenge_bytes(
            &ED25519_BASEPOINT_POINT,
            &y_point,
            &t_point,
            &u_point,
            &r1_point,
            &r2_point,
            &hashlock,
        );

        let c = Scalar::from_bytes_mod_order(challenge);
        let s = k + c * t;
        let mut response = s.to_bytes();
        response[0] ^= 0x42;

        let ok = verify_dleq_proof_bytes(
            &t_point.compress().to_bytes(),
            &u_point.compress().to_bytes(),
            &y_point.compress().to_bytes(),
            &r1_point.compress().to_bytes(),
            &r2_point.compress().to_bytes(),
            &challenge,
            &response,
            &hashlock,
        )
        .unwrap();

        assert!(!ok);
    }

    #[test]
    fn dleq_rejects_bad_point_encoding() {
        let mut rng = OsRng;

        let t = Scalar::random(&mut rng);
        let k = Scalar::random(&mut rng);
        let y = Scalar::random(&mut rng);

        let y_point = &ED25519_BASEPOINT_POINT * &y;
        let t_point = &ED25519_BASEPOINT_POINT * &t;
        let u_point = &y_point * &t;

        let r1_point = &ED25519_BASEPOINT_POINT * &k;
        let r2_point = &y_point * &k;

        let hashlock: [u8; 32] = Sha256::digest(t.to_bytes()).into();

        let challenge = compute_challenge_bytes(
            &ED25519_BASEPOINT_POINT,
            &y_point,
            &t_point,
            &u_point,
            &r1_point,
            &r2_point,
            &hashlock,
        );

        let c = Scalar::from_bytes_mod_order(challenge);
        let s = k + c * t;
        let response = s.to_bytes();

        let bad_point = EdwardsPoint::identity().compress().to_bytes();

        let err = verify_dleq_proof_bytes(
            &bad_point,
            &u_point.compress().to_bytes(),
            &y_point.compress().to_bytes(),
            &r1_point.compress().to_bytes(),
            &r2_point.compress().to_bytes(),
            &challenge,
            &response,
            &hashlock,
        )
        .unwrap_err();

        assert_eq!(err, error!(ErrorCode::InvalidPointEncoding));
    }

    #[test]
    fn challenge_changes_when_inputs_change() {
        let mut rng = OsRng;

        let t = Scalar::random(&mut rng);
        let k = Scalar::random(&mut rng);
        let y = Scalar::random(&mut rng);

        let y_point = &ED25519_BASEPOINT_POINT * &y;
        let t_point = &ED25519_BASEPOINT_POINT * &t;
        let u_point = &y_point * &t;

        let r1_point = &ED25519_BASEPOINT_POINT * &k;
        let r2_point = &y_point * &k;

        let hashlock: [u8; 32] = Sha256::digest(t.to_bytes()).into();

        let base = compute_challenge_bytes(
            &ED25519_BASEPOINT_POINT,
            &y_point,
            &t_point,
            &u_point,
            &r1_point,
            &r2_point,
            &hashlock,
        );

        let mut tweaked_hashlock = hashlock;
        tweaked_hashlock[0] ^= 0x01;

        let tweaked = compute_challenge_bytes(
            &ED25519_BASEPOINT_POINT,
            &y_point,
            &t_point,
            &u_point,
            &r1_point,
            &r2_point,
            &tweaked_hashlock,
        );

        assert_ne!(base, tweaked);
    }
 }

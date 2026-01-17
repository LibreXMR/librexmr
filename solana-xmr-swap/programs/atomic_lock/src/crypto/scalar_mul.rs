use curve25519_dalek::edwards::EdwardsPoint;
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::traits::Identity;

pub fn mul_scalar_small(point: &EdwardsPoint, scalar: &Scalar) -> EdwardsPoint {
    // 4-bit windowed multiplication to reduce compute units.
    let mut table = [EdwardsPoint::identity(); 16];
    table[1] = *point;
    for i in 2..16 {
        table[i] = table[i - 1] + *point;
    }

    let bytes = scalar.to_bytes();
    let mut acc = EdwardsPoint::identity();
    let mut first = true;

    for byte in bytes.iter().rev() {
        let high = byte >> 4;
        let low = byte & 0x0f;
        for nibble in [high, low] {
            if !first {
                acc = acc + acc;
                acc = acc + acc;
                acc = acc + acc;
                acc = acc + acc;
            }
            if nibble != 0 {
                acc = acc + table[nibble as usize];
            }
            first = false;
        }
    }

    acc
}

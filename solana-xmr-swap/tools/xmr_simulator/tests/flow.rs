use tempfile::TempDir;

use xmr_simulator::{compute_hashlock_hex, verify_hashlock, MockWallet};

fn now_unix() -> u64 {
    1_710_000_000
}

#[test]
fn hashlock_roundtrip() {
    let secret = "7496f601de5dbd08c0f65d6927c4570f34a205c37bd98aa8ad83a91241937a0a";
    let hashlock = compute_hashlock_hex(secret).expect("hashlock");
    assert!(verify_hashlock(secret, &hashlock).expect("verify"));
    let bad_secret = "00".repeat(32);
    assert!(!verify_hashlock(&bad_secret, &hashlock).expect("verify"));
}

#[test]
fn lock_and_claim_flow() {
    let temp = TempDir::new().expect("tempdir");
    let wallet = MockWallet::new(temp.path()).expect("wallet");

    let swap_id = "demo-001";
    let secret = "7496f601de5dbd08c0f65d6927c4570f34a205c37bd98aa8ad83a91241937a0a";
    let hashlock = compute_hashlock_hex(secret).expect("hashlock");

    let locked = wallet
        .lock(swap_id, &hashlock, 1_000_000_000_000, now_unix())
        .expect("lock");
    assert_eq!(locked.swap_id, swap_id);
    assert!(locked.claimed_at.is_none());

    let claimed = wallet.claim(swap_id, secret, now_unix()).expect("claim");
    assert_eq!(claimed.swap_id, swap_id);
    assert!(claimed.claimed_at.is_some());
    assert!(claimed.claim_txid.is_some());
}

#[test]
fn claim_rejects_invalid_secret() {
    let temp = TempDir::new().expect("tempdir");
    let wallet = MockWallet::new(temp.path()).expect("wallet");

    let swap_id = "demo-002";
    let secret = "7496f601de5dbd08c0f65d6927c4570f34a205c37bd98aa8ad83a91241937a0a";
    let hashlock = compute_hashlock_hex(secret).expect("hashlock");

    wallet
        .lock(swap_id, &hashlock, 1_000_000_000_000, now_unix())
        .expect("lock");

    let bad_secret = "00".repeat(32);
    let err = wallet
        .claim(swap_id, &bad_secret, now_unix())
        .expect_err("claim should fail");
    assert!(err.to_string().contains("hashlock mismatch"));
}

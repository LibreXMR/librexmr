use swap_coordinator::config::{validate_nonzero, validate_rpc_url};

#[test]
fn rpc_url_requires_http() {
    assert!(validate_rpc_url("rpc", "http://127.0.0.1:8899").is_ok());
    assert!(validate_rpc_url("rpc", "https://api.devnet.solana.com").is_ok());
    assert!(validate_rpc_url("rpc", "ws://localhost").is_err());
    assert!(validate_rpc_url("rpc", "localhost").is_err());
}

#[test]
fn nonzero_validation() {
    assert!(validate_nonzero("lock duration", 1).is_ok());
    assert!(validate_nonzero("lock duration", 0).is_err());
}

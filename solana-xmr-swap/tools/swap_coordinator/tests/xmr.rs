use swap_coordinator::xmr::parse_network;

#[test]
fn parses_valid_networks() {
    assert!(parse_network("mainnet").is_ok());
    assert!(parse_network("testnet").is_ok());
    assert!(parse_network("stagenet").is_ok());
}

#[test]
fn rejects_invalid_network() {
    assert!(parse_network("invalid").is_err());
}

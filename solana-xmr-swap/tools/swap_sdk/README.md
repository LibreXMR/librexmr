# Swap SDK (Client Hardening)

Minimal client-side tool that enforces off-chain DLEQ verification and
persists an audit log of the transcript and verification result.

## Verify and log
```
cargo run -p swap_sdk -- --input test_vectors/dleq.json
```

## Signed audit log
Provide a 32-byte hex signing key (ed25519):
```
cargo run -p swap_sdk -- --input test_vectors/dleq.json \
  --signing-key 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
```

Or load from a file:
```
cargo run -p swap_sdk -- --input test_vectors/dleq.json --signing-key-file key.hex
```

## Verify audit log
```
cargo run -p swap_sdk --bin audit_verify -- --input audit_logs/123_dleq.json
```
Verify the sample signed audit log:
```
cargo run -p swap_sdk --bin audit_verify -- --input docs/audit_signed_sample.json
```

## Write log to stdout
```
cargo run -p swap_sdk -- --input test_vectors/dleq.json --stdout
```

## Custom output path
```
cargo run -p swap_sdk -- --input test_vectors/dleq.json --output audit_logs/custom.json
```

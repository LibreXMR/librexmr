# Webhook Verification

Signed webhook alerts are optional. If a signing key is configured for `swap_ops`,
each alert includes `payload_hash`, `signature`, and `public_key`.

## What Is Signed
The signature is Ed25519 over the JSON bytes of the *payload only*, with the
fields serialized in this exact order and no whitespace:

```
{"event":"state_change","pda":"<PDA>","dleq_verified":true,"unlocked":false,"expired":false,"now_unix":123,"lock_until":456}
```

`payload_hash` is `SHA-256(payload_bytes)` in hex.

## Quick Verification (Recommended)
```
cargo run -p swap_ops --bin alert_verify -- --input alert.json
```

Allow unsigned alerts:
```
cargo run -p swap_ops --bin alert_verify -- --input alert.json --allow-unsigned
```

Sample signed alert:
```
cargo run -p swap_ops --bin alert_verify -- --input docs/alert_signed_sample.json
```

## External Verification Notes
If verifying outside Rust, reconstruct the payload JSON *exactly* as shown
above (same field order, no extra whitespace), compute SHA-256, then verify the
Ed25519 signature with `public_key`. The signature is over the payload bytes,
not the full envelope.

# Swap Ops

Minimal watchtower and status API for swap accounts.

## Build
```
cargo build -p swap_ops
```

## Watch (alerting to stdout)
```
cargo run -p swap_ops -- watch --rpc http://127.0.0.1:8899 --pda <LOCK_PDA>
```

With webhook delivery:
```
cargo run -p swap_ops -- watch --pda <LOCK_PDA> --webhook https://example.com/webhook
```
With signed webhook payloads:
```
cargo run -p swap_ops -- watch --pda <LOCK_PDA> \
  --webhook https://example.com/webhook \
  --webhook-signing-key <hex>
```
Or load the signing key from a file:
```
cargo run -p swap_ops -- watch --pda <LOCK_PDA> \
  --webhook https://example.com/webhook \
  --webhook-signing-key-file key.hex
```
Verify a signed alert (JSON file):
```
cargo run -p swap_ops --bin alert_verify -- --input alert.json
```
Verify the sample signed alert:
```
cargo run -p swap_ops --bin alert_verify -- --input docs/alert_signed_sample.json
```
Allow unsigned alerts:
```
cargo run -p swap_ops --bin alert_verify -- --input alert.json --allow-unsigned
```
You can tune retries/backoff:
```
cargo run -p swap_ops -- watch --pda <LOCK_PDA> \
  --webhook https://example.com/webhook \
  --webhook-retries 3 \
  --webhook-backoff-ms 500
```

You can also provide a file with one PDA per line:
```
cargo run -p swap_ops -- watch --pda-file pdas.txt
```

## Serve (status API)
```
cargo run -p swap_ops -- serve --bind 127.0.0.1:8080 --pda <LOCK_PDA>
```

Endpoints:
- `GET /health`
- `GET /swap/<PDA>`

Alert payload schema is documented in `docs/ALERT_SCHEMA.json`.

Use `--allow-any` to disable the PDA whitelist.

# Integration Checklist

This checklist is for client implementers integrating with the Solana atomic
swap program. It is intended to prevent integration mistakes that would
weaken the security model.

## Required Flow (Happy Path)
1. `initialize`
   - Build and submit the initialization transaction.
   - Confirm it finalized on-chain.
2. Off-chain DLEQ verification (MANDATORY)
   - Verify the full DLEQ proof locally using `tools/dleq_verifier`.
   - Confirm the transcript challenge matches the on-chain rule.
3. `verify_dleq`
   - Only call this after off-chain verification passes.
4. `verify_and_unlock`
   - Provide the correct secret (hashlock preimage).

## Required Validation (Before `verify_dleq`)
- Proof verification passes (full DLEQ math checks).
- Transcript challenge matches the canonical hash rule.
- Points are valid and non‑small‑order (handled by the verifier).
- Hashlock is exactly `SHA-256(secret_bytes)` (no scalar reduction changes).

## Recommended Client Controls
- Refuse to call `verify_dleq` unless the verifier returns OK.
- Log the DLEQ report (computed challenge, r1/r2 checks) for auditing.
- Store the input transcript and verification output with the swap record.

## Reference Tooling
- Verifier CLI:
  ```
  cargo run -p dleq_verifier --bin dleq-verify -- --input test_vectors/dleq.json --verbose
  ```
- TS wrapper:
  ```
  cd tools/dleq_verifier_ts
  npm install
  npm run build
  npm run verify
  ```
- Watchtower / status API:
  ```
  cargo run -p swap_ops -- watch --pda <LOCK_PDA>
  cargo run -p swap_ops -- serve --bind 127.0.0.1:8080 --pda <LOCK_PDA>
  ```
  Webhook alerts:
  ```
  cargo run -p swap_ops -- watch --pda <LOCK_PDA> --webhook https://example.com/webhook
  ```
  Signed webhook alerts:
  ```
  cargo run -p swap_ops -- watch --pda <LOCK_PDA> \
    --webhook https://example.com/webhook \
    --webhook-signing-key <hex>
  ```
  Verify signed alert:
  ```
  cargo run -p swap_ops --bin alert_verify -- --input alert.json
  ```
  Sample signed alert:
  ```
  cargo run -p swap_ops --bin alert_verify -- --input docs/alert_signed_sample.json
  ```
  Schema: `docs/ALERT_SCHEMA.json`
  Verification details: `docs/WEBHOOK_VERIFICATION.md`
- Client hardening (audit log):
  ```
  cargo run -p swap_sdk -- --input test_vectors/dleq.json
  ```
  Signed audit log:
  ```
  cargo run -p swap_sdk -- --input test_vectors/dleq.json --signing-key <hex>
  ```
  Verify audit log:
  ```
  cargo run -p swap_sdk --bin audit_verify -- --input audit_logs/123_dleq.json
  ```
  Sample signed audit log:
  ```
  cargo run -p swap_sdk --bin audit_verify -- --input docs/audit_signed_sample.json
  ```
- Canonical test vector:
  `test_vectors/dleq.json` (mirrored from the Starknet Monero swap repo)

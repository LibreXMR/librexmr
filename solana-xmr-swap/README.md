# Solana XMR Swap (Atomic Lock)

This project implements an atomic swap lock program on Solana using Anchor.

## High-Level Flow
1. `initialize` creates the lock account, validates the DLEQ transcript hash,
   and moves tokens into the PDA vault.
2. Clients verify the **full DLEQ proof off-chain**.
3. `verify_dleq` marks the lock as verified on-chain.
4. `verify_and_unlock` checks the hashlock and releases funds to the unlocker.
5. `refund` returns funds to the depositor after the timelock.

## Off-Chain DLEQ Requirement
The on-chain program does **not** fully verify the DLEQ proof. Clients must
verify it off-chain before calling `verify_dleq`. See `SECURITY.md` for details.

## Architecture & Demo
- Architecture overview: `docs/ARCHITECTURE.md`
- End-to-end demo guide: `docs/DEMO.md`
- Production roadmap: `docs/PRODUCTION.md`
- Hardening checklist: `docs/HARDENING.md`
- Audited libraries: `docs/AUDITED_LIBRARIES.md`

## DLEQ Test Vector
A canonical DLEQ test vector (from the Starknet swap implementation) is stored at
`test_vectors/dleq.json` for client-side verification and compatibility testing.
Source: https://github.com/omarespejel/monero-starknet-atomic-swap

## Off-Chain Verification Tooling
We include a production-grade DLEQ verifier to reduce client integration risk:
```
cargo run -p dleq_verifier --bin dleq-verify -- --input test_vectors/dleq.json --verbose
```
Run unit tests for the verifier:
```
cargo test -p dleq_verifier
```
See `docs/INTEGRATION.md` for the full client checklist.

### TypeScript Wrapper
If you prefer a Node/TS entrypoint:
```
cd tools/dleq_verifier_ts
npm install
npm run build
npm run verify
```

### Watchtower / Status API
Minimal ops tooling lives in `tools/swap_ops`:
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
Verify a signed alert:
```
cargo run -p swap_ops --bin alert_verify -- --input alert.json
```
Alert payload schema: `docs/ALERT_SCHEMA.json`
Webhook verification notes: `docs/WEBHOOK_VERIFICATION.md`
Sample signed alert: `docs/alert_signed_sample.json`

### Client SDK Hardening
The `swap_sdk` tool enforces off-chain DLEQ verification and writes an audit log:
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

### Monero Simulation (demo only)
The Monero claim flow is simulated for hackathon demos:
```
cargo run -p xmr_simulator -- lock --swap-id demo-001 --hashlock <HASHLOCK> --amount 1000000000000
cargo run -p xmr_simulator -- claim --swap-id demo-001 --secret <SECRET>
```

### Monero Wallet (production path)
The production Monero integration is scaffolded in `tools/xmr_wallet` and
uses `monero-rpc` + `monero` (COMIT-aligned libraries). See:
`docs/AUDITED_LIBRARIES.md`.

### Swap Coordinator (ported)
`tools/swap_coordinator` orchestrates a full on-chain swap flow and is **ported and
adapted** from the state machine in the Starknet swap repo by Omar Espejel:
https://github.com/omarespejel/monero-starknet-atomic-swap
```
cargo run -p swap_coordinator -- demo \
  --depositor ~/.config/solana/id.json \
  --amount 1000000000
```

## Audit Bundle
- Toolchain pins: `docs/TOOLCHAIN.md`
- Vendor patch summary: `docs/VENDOR_PATCHES.md`
- Vendor patch hashes: `docs/VENDOR_PATCHES_SHA256.txt`
- Audit bundle index: `docs/AUDIT_BUNDLE.md`
- Audited libraries: `docs/AUDITED_LIBRARIES.md`

## Build (SBF)
If you have the Solana toolchain installed:
```
cargo build-sbf
```

## Frontend
The modern UI lives in `frontend/`:
```
cd frontend
npm install
npm run dev
```
Run tests:
```
npm test
```

## Demo Swap Vector
Generate a fresh demo vector (includes secret) for the UI:
```
cargo run -p dleq_verifier --bin dleq-generate
```
The UI can load it via the “Swap Execution → Load demo swap vector” button.

## Integration Tests (BPF)
The integration tests only run when BPF artifacts are available.
```
BPF_OUT_DIR=target/deploy RUN_BPF_TESTS=1 cargo test -p atomic_lock --test integration
```
Tests also run in CI when `CI=1`.

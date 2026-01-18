# Audit Bundle

This file enumerates the materials needed for a focused audit review.

## Core Documents
- `AUDIT_STATUS.md`
- `SECURITY.md`
- `docs/INTEGRATION.md`
- `docs/ARCHITECTURE.md`
- `docs/DEMO.md`
- `docs/TOOLCHAIN.md`
- `docs/VENDOR_PATCHES.md`
- `docs/VENDOR_PATCHES_SHA256.txt`
- `docs/HARDENING.md`
- `docs/ALERT_SCHEMA.json`
- `docs/WEBHOOK_VERIFICATION.md`
- `docs/AUDITED_LIBRARIES.md`
- `docs/STAGENET_RUNBOOK.md`
- `docs/alert_signed_sample.json`
- `docs/audit_signed_sample.json`

## Canonical Test Vector
- `test_vectors/dleq.json` (mirrored from the Starknet Monero swap repo)
- `test_vectors/dleq_demo.json` (demo vector with secret for local swap)

## Off-Chain Verification Tooling
- Rust verifier: `tools/dleq_verifier`
- TS wrapper: `tools/dleq_verifier_ts`
- Client hardening SDK: `tools/swap_sdk`
- Audit log verifier: `swap_sdk` (`audit-verify` binary)
- Watchtower: `tools/swap_ops` (optional signed webhook alerts)
- Alert verifier: `swap_ops` (`alert-verify` binary)
- Swap coordinator (ported state machine): `tools/swap_coordinator`
- Monero simulator (demo): `tools/xmr_simulator`
- Monero wallet service (production path): `tools/xmr_wallet`

## Verification Commands
```
cargo test -p dleq_verifier
```

```
cd tools/dleq_verifier_ts
npm ci
npm run build
npm run verify
```

## Vendor Patch Hashes
Validate the recorded hashes:
```
shasum -a 256 -c docs/VENDOR_PATCHES_SHA256.txt
```

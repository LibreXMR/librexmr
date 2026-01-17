# Production Roadmap

This document captures the **post‑hackathon** plan for taking the current demo
stack (Solana program + client‑side DLEQ + XMR simulator) to production.

## Scope Split

### Hackathon Scope (ship now)
- Solana program (`atomic_lock`) + end‑to‑end demo flow.
- Client‑side DLEQ verification in SDK + UI.
- Swap coordinator CLI for a one‑shot demo.
- XMR simulator to demonstrate the protocol without a full Monero daemon.
- Architecture and demo docs + video walkthrough.

### Post‑Hackathon / Production
- **Real Monero integration** via official `monero-wallet-rpc` + daemon,
  wrapped by the `monero-rpc` crate.
- **Swap coordinator service** with retries, backoffs, and persistent DB.
- **Key management** (HSM/KMS) + signing key separation.
- **Monitoring** (metrics, alerts, audit trail).
- **Security review** and external audit.

## Production Work Items

1. **Monero wallet service (`tools/xmr_wallet`)**
   - Wrap official `monero-wallet-rpc` via `monero-rpc`.
   - Adaptor completion is a single scalar addition:
     `spend_key = partial_key + revealed_secret` using `curve25519-dalek`.
   - Claim flow uses RPC calls only (no custom wallet crypto).
   - Build confirmation tracking and reorg handling.
   - Store swap metadata and restore heights for efficient sync.

2. **Persistent swap state**
   - Replace JSON files with Postgres/SQLite.
   - Add idempotent transitions and recovery logic.

3. **Key custody & signing**
   - Store secrets and signing keys in HSM/KMS.
   - Rotate keys, track key versions in logs.

4. **Operational hardening**
   - Health checks, alerts, and incident playbooks.
   - Reorg handling on both chains.

5. **Audit + compliance**
   - External audit of on‑chain and off‑chain cryptography.
   - Formal threat model and adversarial tests.

## Auditor-Facing Notes
- Library audit references are tracked in `docs/AUDITED_LIBRARIES.md`.
- Monero has no hashlocks; production swaps use adaptor signatures.

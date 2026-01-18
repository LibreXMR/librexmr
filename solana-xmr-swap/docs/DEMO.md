# Demo Guide

This guide demonstrates an end-to-end swap using local Solana + a simulated
Monero claim. It uses the prebuilt DLEQ demo vector.

## Hackathon Checklist

- [ ] Localnet started and program deployed
- [ ] Demo swap initialized and unlocked via `swap_coordinator`
- [ ] Client-side DLEQ verification completed in UI
- [ ] Helius RPC set via `VITE_HELIUS_RPC` for devnet
- [ ] Helius API key set via `VITE_HELIUS_API_KEY` (tx history panel)
- [ ] XMR simulator claim completed with the revealed secret
- [ ] Audit log or alert verification shown
- [ ] Short demo video recorded

## 0) Demo Script (3 minutes)

**Opening (10s)**
- “LibreXMR is a trustless Solana ↔ Monero atomic swap. No escrow, no KYC.”

**Step 1 — On‑chain lock (30s)**
- Show `swap_coordinator` running on devnet.
- Point out: lock PDA + vault PDA + transaction signature.

**Step 2 — Client‑side DLEQ verification (30s)**
- Open the UI and click “Load demo swap vector.”
- Hit “Verify DLEQ” to show local verification before on‑chain.

**Step 3 — Unlock on Solana (30s)**
- Click “Unlock” and show the devnet tx signature.
- Mention: hashlock checks happen on‑chain.

**Step 4 — Monero claim (simulated) (30s)**
- Run `xmr_simulator` lock + claim with the same secret.
- Show the simulated claim txid.

**Step 4b — Monero claim (stagenet, optional)**
- Run the real `claim-xmr` flow from `docs/STAGENET_RUNBOOK.md`.
- Emphasize this is a real stagenet tx (not a simulator).

**Step 5 — Auditability (20s)**
- Verify a signed alert or audit log with CLI.

**Closing (10s)**
- “We ship the demo now and document the production roadmap.”

## 1) Start Solana localnet

```
solana-test-validator
```

## 2) Build and deploy the program

```
cargo build-sbf
solana program deploy target/deploy/atomic_lock.so
```

## 3) Generate or reuse a demo DLEQ vector

```
cargo run -p dleq_verifier --bin dleq-generate
```

This writes `test_vectors/dleq_demo.json` (includes the secret for demo use).

## 4) Run the swap coordinator (Solana side)

```
cargo run -p swap_coordinator -- demo \
  --depositor ~/.config/solana/id.json \
  --amount 1000000000 \
  --airdrop 2000000000
```

This drives initialize → verify_dleq → unlock using the demo vector.

## 5) Simulate the Monero claim

Use the same secret + hashlock from `test_vectors/dleq_demo.json`:

```
cargo run -p xmr_simulator -- lock \
  --swap-id demo-001 \
  --hashlock <HASHLOCK_HEX> \
  --amount 1000000000000

cargo run -p xmr_simulator -- claim \
  --swap-id demo-001 \
  --secret <SECRET_HEX>
```

The simulator validates the hashlock and prints a fake claim txid.

## 5b) Real Monero claim (stagenet, optional)

Follow `docs/STAGENET_RUNBOOK.md` for a real stagenet claim. Use real
adaptor partials from the swap negotiation (do not use placeholder zeros
outside simulator demos).

## 7) Deploy the frontend (optional)

Render settings:
- Build command: `cd frontend && npm ci && npm run build`
- Publish directory: `frontend/dist`
- Env vars: `VITE_HELIUS_RPC`, `VITE_RPC_URL`, `VITE_HELIUS_API_KEY`

## 6) Frontend demo (optional)

```
cd frontend
npm install
npm run dev
```

Load the demo swap vector in the UI and run **Verify DLEQ** before executing
on-chain steps.

# Release Checklist

Use this list before tagging a release or submitting a demo build.

## Build + Test
- [ ] `make test`
- [ ] `cargo build-sbf`
- [ ] `cd frontend && npm ci && npm run build`
- [ ] `cd frontend && npm run lint`

## Solana Program
- [ ] Program deployed to target cluster (devnet/localnet)
- [ ] Program ID updated in `frontend/src/idl/atomic_lock.ts`
- [ ] IDL matches deployed program

## Frontend
- [ ] `frontend/.env` updated (no secrets committed)
- [ ] `VITE_HELIUS_RPC` and fallback RPC set
- [ ] Optional: `VITE_HELIUS_API_KEY` for Enhanced Transactions

## Monero (Optional Stagenet)
- [ ] `monerod` synced on stagenet
- [ ] `monero-wallet-rpc` running
- [ ] `docs/STAGENET_RUNBOOK.md` completed

## Docs + Audit
- [ ] Demo guide updated (`docs/DEMO.md`)
- [ ] Hardening checklist reviewed (`docs/HARDENING.md`)
- [ ] Audit bundle refreshed (`docs/AUDIT_BUNDLE.md`)

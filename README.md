# LibreXMR

Trustless SOL ↔ XMR atomic swaps. No escrow. No KYC. Just cryptography.

After LocalMonero shut down, users lost the best way to acquire XMR privately.
LibreXMR restores that path with a fully trustless swap between Solana and
Monero.

## What’s in this repo

This repo is organized as a self‑contained project root inside
`solana-xmr-swap/` for production‑grade portability.

- Project root: `solana-xmr-swap/`
- Frontend demo UI: `solana-xmr-swap/frontend/`
- Architecture: `solana-xmr-swap/docs/ARCHITECTURE.md`
- Demo flow: `solana-xmr-swap/docs/DEMO.md`
- Production roadmap: `solana-xmr-swap/docs/PRODUCTION.md`

## Quick Start

```
cd solana-xmr-swap
cargo build -p dleq_verifier
```

Frontend:
```
cd solana-xmr-swap/frontend
npm install
npm run dev
```

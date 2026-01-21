# Swap Coordinator

Orchestrates a full swap flow on Solana (initialize → verify_dleq → unlock),
using the demo DLEQ vector for local testing.

> Ported and adapted from the swap state machine in:
> https://github.com/omarespejel/monero-starknet-atomic-swap (Omar Espejel)

## Demo
```
cargo run -p swap_coordinator -- demo \
  --depositor ~/.config/solana/id.json \
  --amount 1000000000
```

Generate a fresh demo DLEQ vector if needed:
```
cargo run -p dleq_verifier --bin dleq-generate
```

## Setup Mint (optional)
```
cargo run -p swap_coordinator -- setup-mint \
  --depositor ~/.config/solana/id.json \
  --amount 1000000000
```

## RPC Tuning (optional)
Environment overrides for retries/timeouts:
- `SOLANA_RPC_TIMEOUT_MS`
- `SOLANA_RPC_MAX_RETRIES`
- `SOLANA_RPC_BASE_DELAY_MS`
- `SOLANA_RPC_MAX_DELAY_MS`

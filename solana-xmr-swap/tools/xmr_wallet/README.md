# xmr_wallet (Monero Production Path)

This crate provides a production-focused Monero integration using audited
cryptographic primitives and a thin `monero-rpc` wrapper.

## Goals
- Isolate crypto in `src/crypto/` for auditor review.
- Keep RPC calls in `src/rpc/` with no secret handling.
- Use only audited libraries (dalek + noble + RustCrypto).

## Structure
```
src/
├── lib.rs
├── crypto/
│   ├── adaptor.rs
│   └── keys.rs
├── rpc/
│   └── wallet.rs
├── watcher.rs
└── claim.rs
```

## Stagenet Smoke Test
Run Monero wallet RPC:
```
monerod --stagenet --detach
monero-wallet-rpc --stagenet --rpc-bind-port 18083 --wallet-dir ./wallets
```

Then run the ignored test:
```
cargo test -p xmr_wallet --test stagenet -- --ignored
```

You can also run the RPC integration test with environment variables:
```
MONERO_WALLET_RPC_URL=http://127.0.0.1:18083 \
MONERO_WALLET_NAME=audit_test_wallet \
MONERO_WALLET_PASSWORD= \
MONERO_WALLET_LANGUAGE=English \
cargo test -p xmr_wallet --test rpc -- --ignored
```

## Next Milestones
1. Implement lock monitoring in `watcher.rs`.
2. Build claim flow in `claim.rs`.
3. Integrate with `swap_coordinator`.

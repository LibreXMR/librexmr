# Audited & Battle-Tested Libraries

This document provides an auditor-facing view of the cryptographic library
choices and the Monero production path for this project.

## What We're Already Using (Good)

| Library | Status | Used In |
| --- | --- | --- |
| `curve25519-dalek` | Quarkslab audited | On-chain DLEQ transcript checks |
| `@noble/curves` | Cure53 audited | TypeScript DLEQ verification |
| `@noble/hashes` | Cure53 audited | Blake2s transcript hashing |
| `sha2` (RustCrypto) | Widely reviewed | Hashlock computation |

The crypto foundation is solid and aligns with the libraries used in other
production systems.

## COMIT Production Reference

From `xmr-btc-swap`:
https://git.hackliberty.org/Git-Mirrors/xmr-btc-swap/src/commit/62c450192697d955214d0521658dc02ff2e4e1f9

| Component | Library | Audited? |
| --- | --- | --- |
| Monero wallet | `monero-rpc` | Community reviewed |
| Monero transactions | `monero` | Community reviewed |
| Ed25519 / Curve25519 | `curve25519-dalek` | Quarkslab audited |
| Bitcoin | `rust-bitcoin` | Multiple audits |
| Network | `libp2p` | Widely deployed |
| State machine | Custom | Project code |

## Production Stack (Recommended)

## Monero Wallet Options

- **Option 1 (future):** `monero-wallet` + `monero-serai` (audit in progress). Not used yet.
- **Option 2 (alternative):** `eigenwallet/core` (COMIT fork). Not used in this repo.
- **Option 3 (current):** `monero-rpc` wrapper around the official
  `monero-wallet-rpc` binary. This is the safest, lowest-audit-burden path.

### Monero Integration
We use the same Monero libraries as COMIT:

```
[dependencies]
monero-rpc = "0.4"
monero = "0.21"
```

These are wrapped in `tools/xmr_wallet` to keep audited crypto isolated and to
ensure RPC code does not mix with key or signature logic.

All wallet operations are executed by the official `monero-wallet-rpc` daemon;
our code is only a thin RPC wrapper plus a minimal adaptor signature step.

### Monero Service Layout

```
tools/xmr_wallet/
├── src/
│   ├── lib.rs
│   ├── crypto/
│   │   ├── adaptor.rs   # Adaptor signature completion
│   │   └── keys.rs      # Secret/Public key wrappers
│   ├── rpc/
│   │   └── wallet.rs    # monero-wallet-rpc wrapper
│   ├── watcher.rs       # Confirmation tracking (stub)
│   └── claim.rs         # Claim building (stub)
└── Cargo.toml
```

## Critical Security Note: Monero Has No Hashlocks

Monero does not support HTLCs natively. Production swaps use adaptor signatures:

```
Solana side: Hashlock + timelock (on-chain)
Monero side: Adaptor signature (off-chain)
```

The DLEQ proof already verified in this repo ensures the adaptor points share
the same discrete log, enabling a safe claim once the secret is revealed on
Solana.

## Auditor-Facing Notes

- On-chain DLEQ: `curve25519-dalek` (Quarkslab 2019).
- TypeScript DLEQ: `@noble/curves` + `@noble/hashes` (Cure53 2022).
- Monero wallet: official `monero-wallet-rpc` binary (no custom wallet crypto).
- Monero RPC calls: `monero-rpc` crate (community reviewed).
- Pending audits: `monero-wallet` / `monero-serai` (not used yet).
- Custom crypto is limited to one scalar addition for adaptor completion.

## Auditor Checklist (Production)

| Check | Status | Action |
| --- | --- | --- |
| Audited crypto libs | ✅ | dalek + noble |
| Small-order point checks | ✅ | Implemented in dalek wrappers |
| Scalar clamping | ⚠️ | Verify in Monero claim code |
| Timelock enforcement | ✅ | On-chain Solana |
| Secret entropy | ✅ | `OsRng` only |
| Secret logging | ⚠️ | Audit all debug logs |
| Adaptor signature math | ⏳ | Single scalar addition in `xmr_wallet` |
| Confirmation depth | ⏳ | Implement (10+ confs) |
| Reorg handling | ⏳ | Implement |

## Immediate Next Steps

1. Connect to `monero-wallet-rpc` (stagenet) and read balance.
2. Implement adaptor signature completion in `crypto/adaptor.rs`.
3. Add lock/claim monitoring in `watcher.rs`.
4. Integrate `xmr_wallet` into `swap_coordinator`.

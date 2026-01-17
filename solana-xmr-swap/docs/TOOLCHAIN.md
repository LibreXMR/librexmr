# Toolchain Pins

This repository uses a pinned Solana toolchain and a known Rust compiler
version for reproducible builds.

## Rust
- `rustc 1.92.0 (ded5c06cf 2025-12-08)`
- Install/update with `rustup`, then verify:
  ```
  rustc --version
  ```

## Solana (Agave)
- Install via the official installer:
  ```
  sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
  ```
- Version (tested): `v3.1.6`
- Commit: `63057a7b1827d0610f70e36386dcb87f1c23b67a`

## Build Commands
```
cargo build-sbf
```

## Notes
- If you update the Solana toolchain, update this file and re-run integration
  tests (`RUN_BPF_TESTS=1`).

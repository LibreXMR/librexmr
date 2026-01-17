# Security Model

This document describes the security assumptions for the Solana atomic swap
program in this repository.

## Summary
- The on-chain program does **not** fully verify the DLEQ proof.
- The program only checks the transcript hash on-chain and enforces the
  `verify_dleq` step via a flag.
- Full DLEQ verification must be performed **off-chain** by clients.

## What This Means
The cryptographic security of the swap **depends on clients** verifying the
DLEQ proof before proceeding. The chain will not enforce the proof validity.

If a client skips off-chain verification, a malicious counterparty can provide
invalid proof material that still passes the on-chain transcript hash check.
This can break the atomicity and safety properties the DLEQ proof is meant to
guarantee (for example, by creating a swap that cannot be safely redeemed).

This is **not** a cryptographic weakness if clients perform the verification
correctly; it is a shift in responsibility from on-chain to off-chain.

## Required Client Flow
Clients must do the following before calling `verify_dleq`:
1. Fully verify the DLEQ proof off-chain (using a standard implementation).
2. Confirm the transcript challenge matches the expected hash of the points
   and hashlock.
3. Only then call `verify_dleq`, followed by `verify_and_unlock`.

## Reference Material
The canonical DLEQ test vector is sourced from the Starknet Monero atomic swap
implementation and mirrored at `test_vectors/dleq.json` for compatibility testing:
https://github.com/omarespejel/monero-starknet-atomic-swap

## Audit Notes
- This off-chain requirement is intentional and exists due to Solana SBF
  compute limits.
- The trust boundary and responsibility shift must be clearly documented in
  client software and integrator guides.

## Monero Side (Production)
Monero does not support HTLC hashlocks. Production swaps use adaptor
signatures on the Monero side; the DLEQ proof binds the adaptor points so the
secret revealed on Solana enables the Monero claim.

## Known Limitations (Hackathon)
- **Front-running risk on unlock**: the secret is revealed in the
  `verify_and_unlock` transaction. A sophisticated attacker could observe the
  secret in-flight and attempt to front-run with higher fees. This is a known
  limitation for devnet demos.

## Production Hardening
- **Private transaction delivery**: use Jito bundles or private mempools to
  avoid secret extraction/front-running on unlock.
- **Signed alert replay protection**: include a monotonic nonce or UUID in
  webhook payloads for stricter replay defense.
- See `docs/PRODUCTION.md` for the full post-hackathon roadmap.

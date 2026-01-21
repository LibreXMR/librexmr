# Hardening Checklist

This checklist describes the next production hardening steps and how they are
validated with tests. All changes follow TDD.

## Monero RPC Integration (TDD)
- [x] RPC wrapper methods in `tools/xmr_wallet`
- [x] Ignored RPC integration tests (`tests/rpc.rs`)
- [x] Watcher RPC integration test (`tests/watcher_rpc.rs`)
- [ ] Run RPC integration tests in a staging environment

Run the RPC tests locally:
```
MONERO_WALLET_RPC_URL=http://127.0.0.1:18083 \
MONERO_WALLET_NAME=audit_test_wallet \
MONERO_WALLET_PASSWORD= \
MONERO_WALLET_LANGUAGE=English \
make test-rpc
```

## Confirmation Depth + Reorg Handling
- [x] Add confirmation depth config (10+ blocks).
- [x] Track last seen height + reorg buffer in watcher state.
- [x] Tests for rollback on reorg detection.

## Claim Flow Hardening
- [x] Implement adaptor completion path and validate with unit tests.
- [x] Import derived spend key via RPC and sweep funds.
- [x] Negative tests for wrong secret or replay.

## Operational Hardening
- [x] RPC timeouts, retry backoff, and jitter.
  - Implement per-call timeout + max retry budget in `tools/xmr_wallet`.
  - Add unit tests for retry count, backoff growth, and timeout handling.
  - Added Solana RPC timeouts + retries in `swap_coordinator` and frontend fetches.
- [x] Structured logs without secrets.

## Helius Integration
- [x] Document Helius usage in README and demo checklist.
- [x] Add optional Priority Fees API integration.
- [x] Add RPC fallback for priority fees (getRecentPrioritizationFees).
- [x] Add Helius transaction polling with timeouts and safe limits.

## Stagenet Runbook
- [x] Documented end-to-end stagenet runbook (`docs/STAGENET_RUNBOOK.md`).
- [x] Metrics hooks for swap state transitions.
  - Emit counters + latency timers at each transition (init, lock seen, claim, refund).
  - Add a lightweight metrics adapter (stdout/Prometheus) with unit tests.

## Security Review Hooks
- [ ] Static audit checklist run before releases.
- [ ] Regression tests for DLEQ + transcript integrity.

# Hardening Checklist

This checklist describes the next production hardening steps and how they are
validated with tests. All changes follow TDD.

## Monero RPC Integration (TDD)
- [x] RPC wrapper methods in `tools/xmr_wallet`
- [x] Ignored RPC integration tests (`tests/rpc.rs`)
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
- [ ] Implement adaptor completion path and validate with unit tests.
- [ ] Import derived spend key via RPC and sweep funds.
- [ ] Negative tests for wrong secret or replay.

## Operational Hardening
- [ ] RPC timeouts and retry backoff.
- [x] Structured logs without secrets.
- [ ] Metrics hooks for swap state transitions.

## Security Review Hooks
- [ ] Static audit checklist run before releases.
- [ ] Regression tests for DLEQ + transcript integrity.

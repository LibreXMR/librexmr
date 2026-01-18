# Stagenet Runbook (End-to-End)

This runbook drives a full Solana + Monero stagenet flow for the swap demo and
claim integration. It assumes the Solana and Monero toolchains are installed.

## 1) Start Monero Stagenet

```
monerod --stagenet --detach
monero-wallet-rpc --stagenet \
  --rpc-bind-port 18083 \
  --wallet-dir ./wallets \
  --disable-rpc-login
```

## 2) Run Solana Localnet (or Devnet)

Localnet:
```
solana-test-validator
```

Devnet (optional):
```
solana config set --url https://api.devnet.solana.com
```

## 3) Run Solana Demo Swap (on-chain)

```
cargo run -p swap_coordinator -- demo \
  --depositor ~/.config/solana/id.json \
  --amount 1000000000
```

Capture the revealed secret from the demo vector (`test_vectors/dleq_demo.json`).

## 4) Claim XMR (stagenet)

```
cargo run -p swap_coordinator -- claim-xmr \
  --rpc http://127.0.0.1:18083 \
  --network stagenet \
  --wallet claim_wallet \
  --destination <STAGENET_ADDRESS> \
  --alice-partial <ALICE_PARTIAL_HEX> \
  --bob-partial <BOB_PARTIAL_HEX> \
  --secret <REVEALED_SECRET_HEX> \
  --restore-height 0
```

Notes:
- The claim uses adaptor completion and Monero RPC.
- The command will import derived keys and sweep to `--destination`.
- Use real adaptor partials from the swap negotiation (no placeholder zeros).

## 5) Optional: Run RPC Integration Tests

```
MONERO_WALLET_RPC_URL=http://127.0.0.1:18083 \
make test-rpc
```

## Troubleshooting

- If the RPC tests are skipped, ensure `MONERO_WALLET_RPC_URL` is set.
- If stagenet sync is slow, wait for `monerod` to finish syncing before claiming.

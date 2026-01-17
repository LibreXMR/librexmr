# XMR Simulator

Simulates the Monero side of the swap: a hashlocked deposit and claim once the
secret is revealed on-chain.

For the production path (real Monero RPC + adaptor signatures), see
`tools/xmr_wallet`.

## Lock (simulate deposit)
```
cargo run -p xmr_simulator -- lock \
  --swap-id demo-001 \
  --hashlock <SHA256_SECRET_HEX> \
  --amount 1000000000000
```

## Claim (simulate redemption)
```
cargo run -p xmr_simulator -- claim \
  --swap-id demo-001 \
  --secret <SECRET_HEX>
```

## Frontend Environment Example

Copy these into `frontend/.env` locally (do not commit API keys):

```
Copy these lines into `frontend/.env` (and optionally `frontend/.env.example`).

VITE_HELIUS_RPC=https://devnet.helius-rpc.com/?api-key=YOUR_HELIUS_KEY
VITE_QUICKNODE_RPC=https://example.solana-devnet.quiknode.pro/YOUR_KEY/
VITE_RPC_URL=https://api.devnet.solana.com
VITE_HELIUS_API_KEY=YOUR_HELIUS_KEY
VITE_HELIUS_API_URL=https://api.helius.xyz
VITE_HELIUS_TIMEOUT_MS=6000
VITE_HELIUS_TX_POLL_MS=15000
VITE_HELIUS_PRIORITY_LEVEL=Medium
VITE_HELIUS_PRIORITY_RPC_URL=https://mainnet.helius-rpc.com/?api-key=YOUR_HELIUS_KEY
VITE_RPC_RETRY_MAX=2
VITE_RPC_RETRY_BASE_MS=200
VITE_RPC_RETRY_MAX_MS=2000
VITE_RPC_RETRY_JITTER_MS=200
VITE_RANGE_API_KEY=YOUR_RANGE_API_KEY
VITE_RANGE_API_URL=https://api.range.org
VITE_RANGE_FAIL_CLOSED=false
VITE_DEBUG_LOGS=false
```

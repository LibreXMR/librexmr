export const RPC_OPTIONS = [
  {
    label: 'Helius (devnet)',
    value: 'https://devnet.helius-rpc.com/?api-key=YOUR_HELIUS_KEY',
  },
  {
    label: 'QuickNode (devnet)',
    value: 'https://example.solana-devnet.quiknode.pro/YOUR_KEY/',
  },
  {
    label: 'Solana Devnet',
    value: 'https://api.devnet.solana.com',
  },
]

export const DEFAULT_RPC_URL =
  import.meta.env.VITE_RPC_URL ?? RPC_OPTIONS[0].value

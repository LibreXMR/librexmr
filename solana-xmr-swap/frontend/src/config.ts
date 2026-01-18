export const RPC_OPTIONS = [
  {
    label: 'Helius (devnet)',
    value:
      import.meta.env.VITE_HELIUS_RPC ??
      'https://devnet.helius-rpc.com/?api-key=YOUR_HELIUS_KEY',
  },
  {
    label: 'QuickNode (devnet)',
    value:
      import.meta.env.VITE_QUICKNODE_RPC ??
      'https://example.solana-devnet.quiknode.pro/YOUR_KEY/',
  },
  {
    label: 'Solana Devnet',
    value: 'https://api.devnet.solana.com',
  },
]

export const DEFAULT_RPC_URL =
  import.meta.env.VITE_HELIUS_RPC ??
  import.meta.env.VITE_QUICKNODE_RPC ??
  import.meta.env.VITE_RPC_URL ??
  RPC_OPTIONS[0].value

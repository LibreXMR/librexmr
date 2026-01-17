import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import {
  WalletAdapterNetwork,
  WalletError,
} from '@solana/wallet-adapter-base'
import { WalletProvider } from '@solana/wallet-adapter-react'
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui'
import {
  BackpackWalletAdapter,
  PhantomWalletAdapter,
  SolflareWalletAdapter,
} from '@solana/wallet-adapter-wallets'
import { useMemo } from 'react'
import './index.css'
import '@solana/wallet-adapter-react-ui/styles.css'
import App from './App.tsx'

function AppProviders() {
  const network = WalletAdapterNetwork.Devnet
  const wallets = useMemo(
    () => [
      new PhantomWalletAdapter(),
      new SolflareWalletAdapter({ network }),
      new BackpackWalletAdapter(),
    ],
    [network],
  )

  const onError = (error: WalletError) => {
    console.error(error)
  }

  return (
    <WalletProvider wallets={wallets} autoConnect onError={onError}>
      <WalletModalProvider>
        <App />
      </WalletModalProvider>
    </WalletProvider>
  )
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <AppProviders />
  </StrictMode>,
)

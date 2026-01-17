import { useState } from 'react'
import { AlertVerifier } from './sections/AlertVerifier'
import { AuditVerifier } from './sections/AuditVerifier'
import { SwapExecution } from './sections/SwapExecution'
import { SwapBuilder } from './sections/SwapBuilder'
import { SwapStatusView } from './sections/SwapStatus'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'

const tabs = [
  { id: 'swap', label: 'Swap Builder' },
  { id: 'execute', label: 'Swap Execution' },
  { id: 'audit', label: 'Audit Logs' },
  { id: 'alert', label: 'Alert Verify' },
  { id: 'status', label: 'Swap Status' },
] as const

type TabId = (typeof tabs)[number]['id']

export default function App() {
  const [active, setActive] = useState<TabId>('swap')

  return (
    <div className="app">
      <header className="hero">
        <div className="hero-content">
          <span className="eyebrow">Solana ↔ Monero Atomic Swaps</span>
          <h1>Peer-to-peer swaps, locally verified and fully trustless.</h1>
          <p>
            The next LocalMonero-style experience, without escrow risk.
            Every critical step is enforced with cryptography and verifiable
            proofs.
          </p>
          <div className="hero-actions">
            <WalletMultiButton />
          </div>
        </div>
        <div className="hero-panel">
          <div className="panel-header">Trustless by Design</div>
          <ul>
            <li>No custody, no escrow, no counterparty trust</li>
            <li>Off-chain DLEQ verification enforced by clients</li>
            <li>Signed audit logs and signed webhook alerts</li>
          </ul>
        </div>
      </header>

      <section className="positioning">
        <div className="card highlight">
          <div className="card-header">
            <div>
              <h2>Why we’re better than traditional P2P marketplaces</h2>
              <p className="muted">
                LocalMonero-style markets rely on reputation and escrow.
                We remove trust from the equation with atomic swaps.
              </p>
            </div>
          </div>
          <div className="grid two">
            <div className="pill">
              <h3>Trustless settlement</h3>
              <p>
                Funds move only when the cryptographic conditions are satisfied.
                No human escrow, no chargebacks, no arbitration risk.
              </p>
            </div>
            <div className="pill">
              <h3>Local verification</h3>
              <p>
                DLEQ proofs are verified on the client, then pinned on-chain.
                You can prove correctness without revealing secrets.
              </p>
            </div>
            <div className="pill">
              <h3>Operational integrity</h3>
              <p>
                Signed audit logs and signed webhook alerts provide verifiable
                trails for every swap and monitor.
              </p>
            </div>
            <div className="pill">
              <h3>Composable on Solana</h3>
              <p>
                Integrate with wallets, analytics, and ops tooling while keeping
                the privacy-critical path client-side.
              </p>
            </div>
          </div>
        </div>
      </section>

      <nav className="tabs">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            className={active === tab.id ? 'tab active' : 'tab'}
            onClick={() => setActive(tab.id)}
          >
            {tab.label}
          </button>
        ))}
      </nav>

      <main className="content">
        {active === 'swap' && <SwapBuilder />}
        {active === 'execute' && <SwapExecution />}
        {active === 'audit' && <AuditVerifier />}
        {active === 'alert' && <AlertVerifier />}
        {active === 'status' && <SwapStatusView />}
      </main>
    </div>
  )
}

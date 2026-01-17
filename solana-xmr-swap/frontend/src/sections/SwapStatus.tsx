import { useState } from 'react'

type SwapStatus = {
  pda: string
  depositor: string
  hashlock: string
  adaptor_point: string
  second_point: string
  y_point: string
  r1: string
  r2: string
  challenge: string
  response: string
  lock_until: number
  amount: number
  token_mint: string
  vault: string
  dleq_verified: boolean
  unlocked: boolean
  now_unix: number
  expired: boolean
}

export function SwapStatusView() {
  const [baseUrl, setBaseUrl] = useState('http://127.0.0.1:8080')
  const [pda, setPda] = useState('')
  const [status, setStatus] = useState<SwapStatus | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  const fetchStatus = async () => {
    setError(null)
    setStatus(null)
    setLoading(true)
    try {
      const response = await fetch(`${baseUrl}/swap/${pda}`)
      if (!response.ok) {
        throw new Error(`Status request failed (${response.status})`)
      }
      const data = (await response.json()) as SwapStatus
      setStatus(data)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch status')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="card">
      <div className="card-header">
        <div>
          <h2>Swap Status</h2>
          <p className="muted">
            Query swap state via the swap-ops status API.
          </p>
        </div>
        <div className="badge">Monitoring</div>
      </div>

      <div className="grid two">
        <label>
          Status API base URL
          <input
            value={baseUrl}
            onChange={(event) => setBaseUrl(event.target.value)}
            placeholder="http://127.0.0.1:8080"
          />
        </label>
        <label>
          Swap PDA
          <input
            value={pda}
            onChange={(event) => setPda(event.target.value)}
            placeholder="Swap account PDA"
          />
        </label>
      </div>

      <div className="actions">
        <button className="primary" onClick={fetchStatus} disabled={loading}>
          {loading ? 'Fetching...' : 'Fetch status'}
        </button>
      </div>

      {error && <div className="alert error">{error}</div>}

      {status && (
        <div className="output">
          <div className="output-header">
            <span>Swap Status</span>
            <span className={status.unlocked ? 'ok' : 'muted'}>
              {status.unlocked ? 'Unlocked' : 'Locked'}
            </span>
          </div>
          <div className="status-grid">
            <div>
              <span className="label">PDA</span>
              <span className="mono">{status.pda}</span>
            </div>
            <div>
              <span className="label">Token mint</span>
              <span className="mono">{status.token_mint}</span>
            </div>
            <div>
              <span className="label">Amount</span>
              <span>{status.amount}</span>
            </div>
            <div>
              <span className="label">DLEQ verified</span>
              <span>{status.dleq_verified ? 'Yes' : 'No'}</span>
            </div>
            <div>
              <span className="label">Expired</span>
              <span>{status.expired ? 'Yes' : 'No'}</span>
            </div>
            <div>
              <span className="label">Lock until</span>
              <span>{status.lock_until}</span>
            </div>
            <div>
              <span className="label">Hashlock</span>
              <span className="mono">{status.hashlock}</span>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

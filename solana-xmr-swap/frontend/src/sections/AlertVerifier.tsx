import { useState } from 'react'
import { sampleAlert } from '../data/samples'
import { verifyAlertEnvelope, type AlertEnvelope } from '../lib/alert'

export function AlertVerifier() {
  const [raw, setRaw] = useState('')
  const [result, setResult] = useState<ReturnType<typeof verifyAlertEnvelope> | null>(
    null,
  )
  const [error, setError] = useState<string | null>(null)

  const handleVerify = () => {
    setError(null)
    setResult(null)
    try {
      const parsed = JSON.parse(raw) as AlertEnvelope
      const outcome = verifyAlertEnvelope(parsed)
      setResult(outcome)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Invalid JSON')
    }
  }

  const loadSample = () => {
    setRaw(JSON.stringify(sampleAlert, null, 2))
    setResult(null)
    setError(null)
  }

  return (
    <div className="card">
      <div className="card-header">
        <div>
          <h2>Alert Verification</h2>
          <p className="muted">
            Validate signed webhook alerts from the watchtower.
          </p>
        </div>
        <div className="badge">Ops Integrity</div>
      </div>

      <div className="actions">
        <button className="secondary" onClick={loadSample}>
          Load sample
        </button>
      </div>

      <textarea
        className="textarea"
        placeholder="Paste alert JSON"
        rows={10}
        value={raw}
        onChange={(event) => setRaw(event.target.value)}
      />

      <div className="actions">
        <button className="primary" onClick={handleVerify}>
          Verify alert
        </button>
      </div>

      {error && <div className="alert error">{error}</div>}

      {result && (
        <div className="output">
          <div className="output-header">
            <span>Verification Result</span>
            <span className={result.signatureValid ? 'ok' : 'bad'}>
              {result.signatureValid && result.hashMatches ? 'Valid' : 'Invalid'}
            </span>
          </div>
          <div className="status-grid">
            <div>
              <span className="label">Payload hash</span>
              <span className="mono">{result.payloadHash}</span>
            </div>
            <div>
              <span className="label">Hash matches</span>
              <span>{result.hashMatches ? 'Yes' : 'No'}</span>
            </div>
            <div>
              <span className="label">Signature present</span>
              <span>{result.signaturePresent ? 'Yes' : 'No'}</span>
            </div>
            <div>
              <span className="label">Signature valid</span>
              <span>{result.signatureValid ? 'Yes' : 'No'}</span>
            </div>
            {result.error && (
              <div className="error-text">{result.error}</div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}

import { useMemo, useState } from 'react'
import { downloadJson } from '../lib/download'

type SwapForm = {
  depositor: string
  unlocker: string
  tokenMint: string
  amount: string
  lockUntil: string
  hashlock: string
  adaptorPoint: string
  secondPoint: string
  yPoint: string
  r1: string
  r2: string
  challenge: string
  response: string
}

const initialForm: SwapForm = {
  depositor: '',
  unlocker: '',
  tokenMint: '',
  amount: '',
  lockUntil: '',
  hashlock: '',
  adaptorPoint: '',
  secondPoint: '',
  yPoint: '',
  r1: '',
  r2: '',
  challenge: '',
  response: '',
}

export function SwapBuilder() {
  const [form, setForm] = useState<SwapForm>(initialForm)
  const [jsonOut, setJsonOut] = useState<string>('')

  const request = useMemo(() => {
    return {
      depositor: form.depositor.trim(),
      unlocker: form.unlocker.trim(),
      token_mint: form.tokenMint.trim(),
      amount: form.amount.trim(),
      lock_until: form.lockUntil.trim(),
      hashlock: form.hashlock.trim(),
      adaptor_point: form.adaptorPoint.trim(),
      second_point: form.secondPoint.trim(),
      y_point: form.yPoint.trim(),
      r1: form.r1.trim(),
      r2: form.r2.trim(),
      challenge: form.challenge.trim(),
      response: form.response.trim(),
    }
  }, [form])

  const update = (key: keyof SwapForm, value: string) => {
    setForm((prev) => ({ ...prev, [key]: value }))
  }

  const buildJson = () => {
    setJsonOut(JSON.stringify(request, null, 2))
  }

  const reset = () => {
    setForm(initialForm)
    setJsonOut('')
  }

  return (
    <div className="card">
      <div className="card-header">
        <div>
          <h2>Swap Builder</h2>
          <p className="muted">
            Build a swap initialization request to submit with your backend or
            wallet integration.
          </p>
        </div>
        <div className="badge">Solana ↔ Monero</div>
      </div>

      <div className="grid two">
        <label>
          Depositor (Solana pubkey)
          <input
            value={form.depositor}
            onChange={(event) => update('depositor', event.target.value)}
            placeholder="Depositor pubkey"
          />
        </label>
        <label>
          Unlocker (Solana pubkey)
          <input
            value={form.unlocker}
            onChange={(event) => update('unlocker', event.target.value)}
            placeholder="Unlocker pubkey"
          />
        </label>
        <label>
          Token mint
          <input
            value={form.tokenMint}
            onChange={(event) => update('tokenMint', event.target.value)}
            placeholder="Token mint address"
          />
        </label>
        <label>
          Amount (raw)
          <input
            value={form.amount}
            onChange={(event) => update('amount', event.target.value)}
            placeholder="Amount in base units"
          />
        </label>
        <label>
          Lock until (unix)
          <input
            value={form.lockUntil}
            onChange={(event) => update('lockUntil', event.target.value)}
            placeholder="Unix timestamp"
          />
        </label>
        <label>
          Hashlock (SHA-256)
          <input
            value={form.hashlock}
            onChange={(event) => update('hashlock', event.target.value)}
            placeholder="32-byte hex"
          />
        </label>
      </div>

      <div className="grid two">
        <label>
          Adaptor point
          <input
            value={form.adaptorPoint}
            onChange={(event) => update('adaptorPoint', event.target.value)}
            placeholder="Compressed point hex"
          />
        </label>
        <label>
          Second point
          <input
            value={form.secondPoint}
            onChange={(event) => update('secondPoint', event.target.value)}
            placeholder="Compressed point hex"
          />
        </label>
        <label>
          Y point
          <input
            value={form.yPoint}
            onChange={(event) => update('yPoint', event.target.value)}
            placeholder="Compressed point hex"
          />
        </label>
        <label>
          R1
          <input
            value={form.r1}
            onChange={(event) => update('r1', event.target.value)}
            placeholder="Compressed point hex"
          />
        </label>
        <label>
          R2
          <input
            value={form.r2}
            onChange={(event) => update('r2', event.target.value)}
            placeholder="Compressed point hex"
          />
        </label>
        <label>
          Challenge
          <input
            value={form.challenge}
            onChange={(event) => update('challenge', event.target.value)}
            placeholder="Scalar hex"
          />
        </label>
        <label>
          Response
          <input
            value={form.response}
            onChange={(event) => update('response', event.target.value)}
            placeholder="Scalar hex"
          />
        </label>
      </div>

      <div className="actions">
        <button className="primary" onClick={buildJson}>
          Generate JSON
        </button>
        <button
          className="secondary"
          onClick={() => downloadJson('swap_request.json', request)}
        >
          Download JSON
        </button>
        <button className="ghost" onClick={reset}>
          Reset
        </button>
      </div>

      {jsonOut && (
        <div className="output">
          <div className="output-header">
            <span>Swap Request</span>
            <button
              className="link"
              onClick={() => navigator.clipboard.writeText(jsonOut)}
            >
              Copy
            </button>
          </div>
          <pre>{jsonOut}</pre>
        </div>
      )}
    </div>
  )
}

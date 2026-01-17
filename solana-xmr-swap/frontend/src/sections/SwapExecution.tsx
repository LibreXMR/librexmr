import { useMemo, useState } from 'react'
import { useWallet } from '@solana/wallet-adapter-react'
import { PublicKey, Connection } from '@solana/web3.js'
import { demoSwap } from '../data/samples'
import {
  ATOMIC_LOCK_PROGRAM_ID,
} from '../idl/atomic_lock'
import {
  TOKEN_PROGRAM,
  SYSTEM_PROGRAM,
  RENT_SYSVAR,
  deriveLockPda,
  deriveVaultPda,
  ensureAssociatedTokenAccount,
  getProgram,
  parseHex32,
  parseI64,
  parseU64,
} from '../lib/solana'

type FormState = {
  rpcUrl: string
  programId: string
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
  secret: string
}

const initialForm: FormState = {
  rpcUrl: 'http://127.0.0.1:8899',
  programId: ATOMIC_LOCK_PROGRAM_ID.toBase58(),
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
  secret: '',
}

export function SwapExecution() {
  const wallet = useWallet()
  const [form, setForm] = useState<FormState>(initialForm)
  const [status, setStatus] = useState<string[]>([])
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const update = (key: keyof FormState, value: string) => {
    setForm((prev) => ({ ...prev, [key]: value }))
  }

  const connection = useMemo(() => {
    return new Connection(form.rpcUrl, 'confirmed')
  }, [form.rpcUrl])

  const programId = useMemo(() => {
    try {
      return new PublicKey(form.programId)
    } catch {
      return null
    }
  }, [form.programId])

  const derived = useMemo(() => {
    if (!wallet.publicKey || !programId) {
      return { lock: null, vault: null }
    }
    try {
      const hashlock = parseHex32(form.hashlock)
      const [lock] = deriveLockPda(wallet.publicKey, hashlock, programId)
      const [vault] = deriveVaultPda(lock, programId)
      return { lock, vault }
    } catch {
      return { lock: null, vault: null }
    }
  }, [wallet.publicKey, programId, form.hashlock])

  const loadSample = () => {
    setForm((prev) => ({
      ...prev,
      hashlock: demoSwap.hashlock,
      adaptorPoint: demoSwap.adaptorPoint,
      secondPoint: demoSwap.secondPoint,
      yPoint: demoSwap.yPoint,
      r1: demoSwap.r1,
      r2: demoSwap.r2,
      challenge: demoSwap.challenge,
      response: demoSwap.response,
      secret: demoSwap.secret,
    }))
    setError(null)
  }

  const pushStatus = (message: string) => {
    setStatus((prev) => [message, ...prev].slice(0, 6))
  }

  const execute = async (fn: () => Promise<void>) => {
    setError(null)
    setBusy(true)
    try {
      await fn()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Transaction failed')
    } finally {
      setBusy(false)
    }
  }

  const initializeSwap = async () => {
    if (!wallet.publicKey) {
      throw new Error('Connect a wallet first')
    }
    if (!programId) {
      throw new Error('Invalid program ID')
    }
    const tokenMint = new PublicKey(form.tokenMint)
    const hashlock = parseHex32(form.hashlock)
    const adaptorPoint = parseHex32(form.adaptorPoint)
    const secondPoint = parseHex32(form.secondPoint)
    const yPoint = parseHex32(form.yPoint)
    const r1 = parseHex32(form.r1)
    const r2 = parseHex32(form.r2)
    const challenge = parseHex32(form.challenge)
    const response = parseHex32(form.response)
    const lockUntil = parseI64(form.lockUntil)
    const amount = parseU64(form.amount)

    const program = getProgram(connection, wallet, programId)
    const [lockPda] = deriveLockPda(wallet.publicKey, hashlock, programId)
    const [vaultPda] = deriveVaultPda(lockPda, programId)

    const { ata, ix } = await ensureAssociatedTokenAccount(
      connection,
      wallet.publicKey,
      tokenMint,
      wallet.publicKey,
    )

    const builder = program.methods
      .initialize(
        hashlock,
        adaptorPoint,
        secondPoint,
        yPoint,
        r1,
        r2,
        challenge,
        response,
        lockUntil,
        amount,
      )
      .accounts({
        depositor: wallet.publicKey,
        atomicLock: lockPda,
        vault: vaultPda,
        depositorToken: ata,
        tokenMint,
        tokenProgram: TOKEN_PROGRAM,
        systemProgram: SYSTEM_PROGRAM,
        rent: RENT_SYSVAR,
      })

    const signature = await builder
      .preInstructions(ix ? [ix] : [])
      .rpc()
    pushStatus(`Initialized swap: ${signature}`)
  }

  const verifyDleq = async () => {
    if (!wallet.publicKey) {
      throw new Error('Connect a wallet first')
    }
    if (!programId || !derived.lock) {
      throw new Error('Missing PDA or program ID')
    }
    const program = getProgram(connection, wallet, programId)
    const signature = await program.methods
      .verifyDleq()
      .accounts({
        atomicLock: derived.lock,
      })
      .rpc()
    pushStatus(`DLEQ verified: ${signature}`)
  }

  const unlockSwap = async () => {
    if (!wallet.publicKey) {
      throw new Error('Connect a wallet first')
    }
    if (!programId || !derived.lock || !derived.vault) {
      throw new Error('Missing PDA or program ID')
    }
    const tokenMint = new PublicKey(form.tokenMint)
    const secret = parseHex32(form.secret)
    const program = getProgram(connection, wallet, programId)

    const { ata, ix } = await ensureAssociatedTokenAccount(
      connection,
      wallet.publicKey,
      tokenMint,
      wallet.publicKey,
    )

    const signature = await program.methods
      .verifyAndUnlock(secret)
      .accounts({
        unlocker: wallet.publicKey,
        atomicLock: derived.lock,
        vault: derived.vault,
        unlockerToken: ata,
        tokenProgram: TOKEN_PROGRAM,
      })
      .preInstructions(ix ? [ix] : [])
      .rpc()
    pushStatus(`Unlocked swap: ${signature}`)
  }

  const refundSwap = async () => {
    if (!wallet.publicKey) {
      throw new Error('Connect a wallet first')
    }
    if (!programId || !derived.lock || !derived.vault) {
      throw new Error('Missing PDA or program ID')
    }
    const tokenMint = new PublicKey(form.tokenMint)
    const program = getProgram(connection, wallet, programId)
    const { ata, ix } = await ensureAssociatedTokenAccount(
      connection,
      wallet.publicKey,
      tokenMint,
      wallet.publicKey,
    )
    const signature = await program.methods
      .refund()
      .accounts({
        depositor: wallet.publicKey,
        atomicLock: derived.lock,
        vault: derived.vault,
        depositorToken: ata,
        tokenProgram: TOKEN_PROGRAM,
      })
      .preInstructions(ix ? [ix] : [])
      .rpc()
    pushStatus(`Refunded swap: ${signature}`)
  }

  return (
    <div className="card">
      <div className="card-header">
        <div>
          <h2>Swap Execution</h2>
          <p className="muted">
            Connect a wallet and run the on-chain steps for a trustless swap.
          </p>
        </div>
        <div className="badge">On-chain Flow</div>
      </div>

      <div className="status-line">
        <span className="label">Wallet</span>
        <span className="mono">
          {wallet.publicKey ? wallet.publicKey.toBase58() : 'Not connected'}
        </span>
      </div>

      <div className="alert info">
        Requires a funded token account for the selected mint and a deployed
        program on the target RPC (localnet or devnet).
      </div>

      <div className="grid two">
        <label>
          RPC URL
          <input
            value={form.rpcUrl}
            onChange={(event) => update('rpcUrl', event.target.value)}
          />
        </label>
        <label>
          Program ID
          <input
            value={form.programId}
            onChange={(event) => update('programId', event.target.value)}
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

      <div className="actions">
        <button className="secondary" onClick={loadSample}>
          Load demo swap vector
        </button>
      </div>

      <div className="grid two">
        <label>
          Adaptor point
          <input
            value={form.adaptorPoint}
            onChange={(event) => update('adaptorPoint', event.target.value)}
          />
        </label>
        <label>
          Second point
          <input
            value={form.secondPoint}
            onChange={(event) => update('secondPoint', event.target.value)}
          />
        </label>
        <label>
          Y point
          <input
            value={form.yPoint}
            onChange={(event) => update('yPoint', event.target.value)}
          />
        </label>
        <label>
          R1
          <input value={form.r1} onChange={(event) => update('r1', event.target.value)} />
        </label>
        <label>
          R2
          <input value={form.r2} onChange={(event) => update('r2', event.target.value)} />
        </label>
        <label>
          Challenge
          <input
            value={form.challenge}
            onChange={(event) => update('challenge', event.target.value)}
          />
        </label>
        <label>
          Response
          <input
            value={form.response}
            onChange={(event) => update('response', event.target.value)}
          />
        </label>
      </div>

      <div className="grid two">
        <label>
          Secret (hashlock preimage)
          <input
            value={form.secret}
            onChange={(event) => update('secret', event.target.value)}
            placeholder="32-byte hex"
          />
        </label>
        <label>
          Derived Lock PDA
          <input value={derived.lock?.toBase58() ?? ''} readOnly />
        </label>
        <label>
          Derived Vault PDA
          <input value={derived.vault?.toBase58() ?? ''} readOnly />
        </label>
      </div>

      <div className="actions">
        <button className="primary" disabled={busy} onClick={() => execute(initializeSwap)}>
          Initialize
        </button>
        <button className="secondary" disabled={busy} onClick={() => execute(verifyDleq)}>
          Verify DLEQ
        </button>
        <button className="secondary" disabled={busy} onClick={() => execute(unlockSwap)}>
          Unlock
        </button>
        <button className="ghost" disabled={busy} onClick={() => execute(refundSwap)}>
          Refund
        </button>
      </div>

      {error && <div className="alert error">{error}</div>}

      {status.length > 0 && (
        <div className="output">
          <div className="output-header">
            <span>Recent Activity</span>
          </div>
          <ul className="status-list">
            {status.map((entry) => (
              <li key={entry}>{entry}</li>
            ))}
          </ul>
        </div>
      )}
    </div>
  )
}

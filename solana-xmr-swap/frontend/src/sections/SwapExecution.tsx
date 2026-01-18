import { useCallback, useEffect, useMemo, useState } from 'react'
import { useWallet } from '@solana/wallet-adapter-react'
import { PublicKey, Connection, ComputeBudgetProgram } from '@solana/web3.js'
import { demoSwap } from '../data/samples'
import { DEFAULT_RPC_URL, RPC_OPTIONS } from '../config'
import { verifyDleqClientSide } from '../lib/dleq'
import { downloadJson } from '../lib/download'
import {
  fetchHeliusTransactions,
  isHeliusEnabled,
  type HeliusTransaction,
} from '../lib/helius'
import { screenSwapParties } from '../lib/compliance'
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
  fetchPriorityFeeEstimate,
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
  recipient: string
}

type StatusEntry = {
  message: string
  signature?: string
}

const initialForm: FormState = {
  rpcUrl: DEFAULT_RPC_URL,
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
  recipient: '',
}

export function SwapExecution() {
  const wallet = useWallet()
  const [form, setForm] = useState<FormState>(initialForm)
  const [status, setStatus] = useState<StatusEntry[]>([])
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [heliusTxs, setHeliusTxs] = useState<HeliusTransaction[]>([])
  const [heliusError, setHeliusError] = useState<string | null>(null)
  const [complianceEnabled, setComplianceEnabled] = useState(true)
  const [complianceStatus, setComplianceStatus] = useState<
    'idle' | 'checking' | 'pass' | 'fail' | 'skipped'
  >('idle')

  const debugEnabled =
    (import.meta.env.VITE_DEBUG_LOGS as string | undefined)?.toLowerCase() === 'true'
  const debugLog = useCallback((...args: unknown[]) => {
    if (debugEnabled) {
      console.debug('[swap]', ...args)
    }
  }, [debugEnabled])
  const debugWarn = useCallback((...args: unknown[]) => {
    if (debugEnabled) {
      console.warn('[swap]', ...args)
    }
  }, [debugEnabled])
  const redactHex = useCallback((value: string, keep = 8) => {
    const trimmed = value.trim()
    if (!trimmed) return ''
    if (trimmed.length <= keep * 2) return trimmed
    return `${trimmed.slice(0, keep)}...${trimmed.slice(-4)}`
  }, [])

  const update = (key: keyof FormState, value: string) => {
    setForm((prev) => ({ ...prev, [key]: value }))
  }

  const connection = useMemo(() => {
    return new Connection(form.rpcUrl, 'confirmed')
  }, [form.rpcUrl])

  const programIdInput = useMemo(() => form.programId.trim(), [form.programId])

  const programId = useMemo(() => {
    if (!programIdInput) {
      return null
    }
    try {
      return new PublicKey(programIdInput)
    } catch {
      return null
    }
  }, [programIdInput])

  const programIdError = useMemo(() => {
    if (!programIdInput) {
      return 'Program ID is required'
    }
    try {
      new PublicKey(programIdInput)
      return null
    } catch {
      return 'Program ID is invalid'
    }
  }, [programIdInput])

  const hashlockInput = useMemo(() => form.hashlock.trim(), [form.hashlock])

  const hashlockError = useMemo(() => {
    if (!hashlockInput) {
      return 'Hashlock is required'
    }
    if (!/^[0-9a-fA-F]+$/.test(hashlockInput)) {
      return 'Hashlock must be hex'
    }
    if (hashlockInput.length !== 64) {
      return 'Hashlock must be 64 hex chars'
    }
    return null
  }, [hashlockInput])

  const derived = useMemo(() => {
    if (!wallet.publicKey || !programId) {
      return { lock: null, vault: null }
    }
    try {
      const hashlock = parseHex32(hashlockInput)
      const [lock] = deriveLockPda(wallet.publicKey, hashlock, programId)
      const [vault] = deriveVaultPda(lock, programId)
      return { lock, vault }
    } catch {
      return { lock: null, vault: null }
    }
  }, [wallet.publicKey, programId, hashlockInput])

  const pdaError = useMemo(() => {
    if (!wallet.publicKey) {
      return 'Wallet not connected'
    }
    if (!programId) {
      return programIdError
    }
    if (hashlockError) {
      return hashlockError
    }
    return null
  }, [wallet.publicKey, programId, programIdError, hashlockError])

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
  debugLog('Loaded demo swap vector', {
    hashlock: redactHex(demoSwap.hashlock),
    adaptorPoint: redactHex(demoSwap.adaptorPoint),
    secondPoint: redactHex(demoSwap.secondPoint),
    yPoint: redactHex(demoSwap.yPoint),
  })
  }

  const pushStatus = (message: string, signature?: string) => {
    setStatus((prev) => [{ message, signature }, ...prev].slice(0, 6))
  }

  const buildPriorityFeeIx = async (accountKeys: PublicKey[]) => {
    const { estimate, source } = await fetchPriorityFeeEstimate(
      form.rpcUrl,
      accountKeys,
      connection,
    )
    if (!estimate || estimate <= 0) {
      debugLog('Priority fee estimate unavailable', { source, estimate })
      return null
    }
    const sourceLabel = source === 'helius' ? 'Helius' : 'RPC fallback'
    pushStatus(
      `Priority fee estimate (${sourceLabel}): ${Math.round(estimate)} μ-lamports/CU`,
    )
    debugLog('Priority fee estimate', { source, estimate })
    return ComputeBudgetProgram.setComputeUnitPrice({
      microLamports: Math.ceil(estimate),
    })
  }

  useEffect(() => {
    if (!derived.lock || !isHeliusEnabled(form.rpcUrl)) {
      setHeliusTxs([])
      setHeliusError(null)
      return
    }
    let cancelled = false
    const pollMs = Number(import.meta.env.VITE_HELIUS_TX_POLL_MS ?? 15000)
    const poll = async () => {
      try {
        debugLog('Helius poll start', {
          lockPda: derived.lock.toBase58(),
          rpcUrl: form.rpcUrl,
        })
        const txs = await fetchHeliusTransactions(
          form.rpcUrl,
          derived.lock.toBase58(),
          10,
        )
        if (!cancelled) {
          setHeliusTxs(txs)
          setHeliusError(null)
        }
      } catch (err) {
        if (!cancelled) {
          debugWarn('Helius poll failed', err)
          setHeliusError(
            err instanceof Error ? err.message : 'Failed to load Helius history',
          )
        }
      }
    }
    poll()
    const interval = window.setInterval(poll, pollMs)
    return () => {
      cancelled = true
      window.clearInterval(interval)
    }
  }, [derived.lock, form.rpcUrl, debugLog, debugWarn])

  useEffect(() => {
    debugLog('Derived PDA state', {
      wallet: wallet.publicKey?.toBase58() ?? null,
      rpcUrl: form.rpcUrl,
      programId: programId?.toBase58() ?? null,
      programIdInput,
      programIdError,
      hashlockInput,
      hashlockError,
      hashlockLength: hashlockInput.length,
      lockPda: derived.lock?.toBase58() ?? null,
      vaultPda: derived.vault?.toBase58() ?? null,
      pdaError,
    })
  }, [
    wallet.publicKey,
    form.rpcUrl,
    programId,
    programIdInput,
    programIdError,
    hashlockInput,
    hashlockError,
    derived.lock,
    derived.vault,
    pdaError,
    debugLog,
  ])

  const execute = async (label: string, fn: () => Promise<void>) => {
    setError(null)
    setBusy(true)
    debugLog('Execute action start', { label })
    try {
      await fn()
    } catch (err) {
      debugWarn('Execution error', err)
      setError(err instanceof Error ? err.message : 'Transaction failed')
    } finally {
      debugLog('Execute action end', { label })
      setBusy(false)
    }
  }

  const recipientInfo = useMemo(() => {
    const trimmed = form.recipient.trim()
    if (!trimmed) {
      return { valid: true, key: wallet.publicKey ?? null }
    }
    try {
      return { valid: true, key: new PublicKey(trimmed) }
    } catch {
      return { valid: false, key: null }
    }
  }, [form.recipient, wallet.publicKey])

  const initializeSwap = async () => {
    if (!wallet.publicKey) {
      throw new Error('Connect a wallet first')
    }
    if (!programId) {
      throw new Error('Invalid program ID')
    }
    if (form.recipient.trim() && !recipientInfo.valid) {
      throw new Error('Invalid recipient address')
    }

    debugLog('Initialize swap inputs', {
      rpcUrl: form.rpcUrl,
      programId: programId.toBase58(),
      tokenMint: form.tokenMint,
      amount: form.amount,
      lockUntil: form.lockUntil,
      hashlock: redactHex(form.hashlock),
      adaptorPoint: redactHex(form.adaptorPoint),
      secondPoint: redactHex(form.secondPoint),
      yPoint: redactHex(form.yPoint),
      r1: redactHex(form.r1),
      r2: redactHex(form.r2),
      challenge: redactHex(form.challenge),
      response: redactHex(form.response),
      recipient: recipientInfo.key?.toBase58() ?? null,
    })

    if (complianceEnabled) {
      setComplianceStatus('checking')
      try {
        const compliance = await screenSwapParties(
          wallet.publicKey.toBase58(),
          recipientInfo.key?.toBase58(),
        )
        debugLog('Compliance result', compliance)
        const failed =
          !compliance.depositor.isClean ||
          (compliance.recipient && !compliance.recipient.isClean)
        const skipped =
          compliance.depositor.source === 'skipped' &&
          (!compliance.recipient || compliance.recipient.source === 'skipped')
        if (failed) {
          setComplianceStatus('fail')
          throw new Error('Compliance screening failed')
        }
        setComplianceStatus(skipped ? 'skipped' : 'pass')
      } catch (err) {
        if (err instanceof Error && err.message === 'Compliance screening failed') {
          throw err
        }
        debugWarn('Compliance check skipped', err)
        setComplianceStatus('skipped')
      }
    } else {
      debugLog('Compliance disabled')
      setComplianceStatus('skipped')
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

    const computeIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })
    const priorityIx = await buildPriorityFeeIx([
      wallet.publicKey,
      lockPda,
      vaultPda,
      tokenMint,
      programId,
    ])
    const signature = await builder
      .preInstructions(
        ix
          ? [computeIx, ...(priorityIx ? [priorityIx] : []), ix]
          : [computeIx, ...(priorityIx ? [priorityIx] : [])],
      )
      .rpc()
    debugLog('Initialize tx', { signature })
    pushStatus('Initialized swap', signature)
  }

  const verifyDleq = async () => {
    if (!wallet.publicKey) {
      throw new Error('Connect a wallet first')
    }
    debugLog('Verify DLEQ input', {
      hashlock: redactHex(form.hashlock),
      adaptorPoint: redactHex(form.adaptorPoint),
      secondPoint: redactHex(form.secondPoint),
      yPoint: redactHex(form.yPoint),
      r1: redactHex(form.r1),
      r2: redactHex(form.r2),
      challenge: redactHex(form.challenge),
      response: redactHex(form.response),
    })
    const local = verifyDleqClientSide({
      adaptorPoint: form.adaptorPoint,
      secondPoint: form.secondPoint,
      yPoint: form.yPoint,
      r1: form.r1,
      r2: form.r2,
      challenge: form.challenge,
      response: form.response,
      hashlock: form.hashlock,
    })
    if (!local.ok) {
      throw new Error(
        `Local DLEQ verification failed (challengeMatches=${local.report.challengeMatches}, lhsR1Matches=${local.report.lhsR1Matches}, lhsR2Matches=${local.report.lhsR2Matches})`,
      )
    }
    debugLog('Local DLEQ verified', local.report)
    pushStatus('Local DLEQ verified')
    if (!wallet.publicKey || !programId || !derived.lock) {
      pushStatus(`On-chain verify skipped: ${pdaError ?? 'missing PDA or program ID'}`)
      return
    }
    const program = getProgram(connection, wallet, programId)
    const computeIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })
    const priorityIx = await buildPriorityFeeIx([
      wallet.publicKey,
      derived.lock,
      programId,
    ])
    const signature = await program.methods
      .verifyDleq()
      .accounts({
        atomicLock: derived.lock,
      })
      .preInstructions([computeIx, ...(priorityIx ? [priorityIx] : [])])
      .rpc()
    debugLog('Verify DLEQ tx', { signature })
    pushStatus('DLEQ verified', signature)
  }

  const unlockSwap = async () => {
    if (!wallet.publicKey) {
      throw new Error('Connect a wallet first')
    }
    if (!programId || !derived.lock || !derived.vault) {
      throw new Error('Missing PDA or program ID')
    }
    debugLog('Unlock inputs', {
      lockPda: derived.lock.toBase58(),
      vaultPda: derived.vault.toBase58(),
      tokenMint: form.tokenMint,
      secret: redactHex(form.secret),
      recipient: recipientInfo.key?.toBase58() ?? null,
    })
    const tokenMint = new PublicKey(form.tokenMint)
    const secret = parseHex32(form.secret)
    const program = getProgram(connection, wallet, programId)
    if (!recipientInfo.valid || !recipientInfo.key) {
      throw new Error('Recipient address is invalid')
    }
    const recipient = recipientInfo.key

    const { ata, ix } = await ensureAssociatedTokenAccount(
      connection,
      wallet.publicKey,
      tokenMint,
      recipient,
    )

    const computeIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })
    const priorityIx = await buildPriorityFeeIx([
      wallet.publicKey,
      derived.lock,
      derived.vault,
      tokenMint,
      recipient,
      programId,
    ])
    const signature = await program.methods
      .verifyAndUnlock(secret)
      .accounts({
        unlocker: wallet.publicKey,
        atomicLock: derived.lock,
        vault: derived.vault,
        unlockerToken: ata,
        tokenProgram: TOKEN_PROGRAM,
      })
      .preInstructions(
        ix
          ? [computeIx, ...(priorityIx ? [priorityIx] : []), ix]
          : [computeIx, ...(priorityIx ? [priorityIx] : [])],
      )
      .rpc()
    debugLog('Unlock tx', { signature })
    pushStatus('Unlocked swap', signature)
  }

  const refundSwap = async () => {
    if (!wallet.publicKey) {
      throw new Error('Connect a wallet first')
    }
    if (!programId || !derived.lock || !derived.vault) {
      throw new Error('Missing PDA or program ID')
    }
    debugLog('Refund inputs', {
      lockPda: derived.lock.toBase58(),
      vaultPda: derived.vault.toBase58(),
      tokenMint: form.tokenMint,
    })
    const tokenMint = new PublicKey(form.tokenMint)
    const program = getProgram(connection, wallet, programId)
    const { ata, ix } = await ensureAssociatedTokenAccount(
      connection,
      wallet.publicKey,
      tokenMint,
      wallet.publicKey,
    )
    const computeIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })
    const priorityIx = await buildPriorityFeeIx([
      wallet.publicKey,
      derived.lock,
      derived.vault,
      tokenMint,
      programId,
    ])
    const signature = await program.methods
      .refund()
      .accounts({
        depositor: wallet.publicKey,
        atomicLock: derived.lock,
        vault: derived.vault,
        depositorToken: ata,
        tokenProgram: TOKEN_PROGRAM,
      })
      .preInstructions(
        ix
          ? [computeIx, ...(priorityIx ? [priorityIx] : []), ix]
          : [computeIx, ...(priorityIx ? [priorityIx] : [])],
      )
      .rpc()
    debugLog('Refund tx', { signature })
    pushStatus('Refunded swap', signature)
  }

  const buildAuditSnapshot = () => ({
    timestamp: new Date().toISOString(),
    rpcUrl: form.rpcUrl,
    programId: form.programId,
    lockPda: derived.lock?.toBase58() ?? null,
    vaultPda: derived.vault?.toBase58() ?? null,
    recipient: form.recipient.trim() || wallet.publicKey?.toBase58() || null,
    form: {
      tokenMint: form.tokenMint,
      amount: form.amount,
      lockUntil: form.lockUntil,
      hashlock: form.hashlock,
      adaptorPoint: form.adaptorPoint,
      secondPoint: form.secondPoint,
      yPoint: form.yPoint,
      r1: form.r1,
      r2: form.r2,
      challenge: form.challenge,
      response: form.response,
    },
    localDleq: (() => {
      try {
        return verifyDleqClientSide({
          adaptorPoint: form.adaptorPoint,
          secondPoint: form.secondPoint,
          yPoint: form.yPoint,
          r1: form.r1,
          r2: form.r2,
          challenge: form.challenge,
          response: form.response,
          hashlock: form.hashlock,
        })
      } catch (err) {
        return {
          ok: false,
          report: {
            computedChallenge: null,
            challengeMatches: false,
            lhsR1Matches: false,
            lhsR2Matches: false,
          },
          error: err instanceof Error ? err.message : 'Unknown error',
        }
      }
    })(),
    status,
  })

  const exportAudit = () => {
    debugLog('Export audit', {
      rpcUrl: form.rpcUrl,
      programId: programId?.toBase58() ?? null,
      lockPda: derived.lock?.toBase58() ?? null,
    })
    downloadJson('swap-audit.json', buildAuditSnapshot())
  }

  const copyAudit = async () => {
    const payload = JSON.stringify(buildAuditSnapshot(), null, 2)
    if (!navigator.clipboard?.writeText) {
      throw new Error('Clipboard API unavailable')
    }
    await navigator.clipboard.writeText(payload)
    debugLog('Audit JSON copied')
    pushStatus('Audit JSON copied to clipboard')
  }

  const explorerCluster = useMemo(() => {
    const url = form.rpcUrl.toLowerCase()
    if (url.includes('devnet')) return 'devnet'
    if (url.includes('testnet')) return 'testnet'
    return 'mainnet-beta'
  }, [form.rpcUrl])

  const heliusEnabled = useMemo(
    () => isHeliusEnabled(form.rpcUrl),
    [form.rpcUrl],
  )

  const explorerUrl = (type: 'tx' | 'address', value: string) => {
    const base =
      type === 'tx'
        ? `https://explorer.solana.com/tx/${value}`
        : `https://explorer.solana.com/address/${value}`
    return `${base}?cluster=${explorerCluster}`
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
          RPC Provider
          <select
            value={form.rpcUrl}
            onChange={(event) => update('rpcUrl', event.target.value)}
          >
            {RPC_OPTIONS.map((option) => (
              <option key={option.label} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </label>
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
          {programIdError && (
            <span className="muted">{programIdError}</span>
          )}
          {programId && (
            <a
              className="muted"
              href={explorerUrl('address', programId.toBase58())}
              target="_blank"
              rel="noreferrer"
            >
              View program
            </a>
          )}
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
          {hashlockError && (
            <span className="muted">{hashlockError}</span>
          )}
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
          Recipient (unlock destination)
          <input
            value={form.recipient}
            onChange={(event) => update('recipient', event.target.value)}
            placeholder="Defaults to connected wallet"
          />
          {!recipientInfo.valid && form.recipient.trim() && (
            <span className="muted">Invalid recipient address</span>
          )}
        </label>
        <label>
          Compliance checks
          <input
            type="checkbox"
            checked={complianceEnabled}
            onChange={(event) => {
              setComplianceEnabled(event.target.checked)
              setComplianceStatus('idle')
            }}
          />
          <span className="muted">
            Toggle Range screening for demos.
          </span>
        </label>
        <label>
          Derived Lock PDA
          <input value={derived.lock?.toBase58() ?? ''} readOnly />
          {!derived.lock && pdaError && (
            <span className="muted">{pdaError}</span>
          )}
        </label>
        <label>
          Derived Vault PDA
          <input value={derived.vault?.toBase58() ?? ''} readOnly />
        </label>
      </div>

      <div className="actions">
        <button
          className="primary"
          disabled={busy}
          onClick={() => execute('initialize', initializeSwap)}
        >
          Initialize
        </button>
        <button
          className="secondary"
          disabled={busy}
          onClick={() => execute('verify_dleq', verifyDleq)}
        >
          Verify DLEQ
        </button>
        <button
          className="secondary"
          disabled={busy}
          onClick={() => execute('unlock', unlockSwap)}
        >
          Unlock
        </button>
        <button
          className="ghost"
          disabled={busy}
          onClick={() => execute('refund', refundSwap)}
        >
          Refund
        </button>
        <button className="ghost" disabled={busy} onClick={exportAudit}>
          Export Audit
        </button>
        <button
          className="ghost"
          disabled={busy}
          onClick={() => execute('copy_audit', copyAudit)}
        >
          Copy Audit JSON
        </button>
      </div>

      {error && <div className="alert error">{error}</div>}

      {complianceStatus !== 'idle' && (
        <div className="alert info">
          {complianceStatus === 'checking' && 'Compliance: checking...'}
          {complianceStatus === 'pass' && 'Compliance: passed'}
          {complianceStatus === 'skipped' && 'Compliance: skipped (no API key)'}
          {complianceStatus === 'fail' && 'Compliance: failed'}
        </div>
      )}

      {status.length > 0 && (
        <div className="output">
          <div className="output-header">
            <span>Recent Activity</span>
          </div>
          <ul className="status-list">
            {status.map((entry, idx) => (
              <li key={`${entry.message}-${idx}`}>
                {entry.message}
                {entry.signature ? (
                  <>
                    {' '}
                    <a
                      className="muted"
                      href={explorerUrl('tx', entry.signature)}
                      target="_blank"
                      rel="noreferrer"
                    >
                      View tx
                    </a>
                  </>
                ) : null}
              </li>
            ))}
          </ul>
        </div>
      )}

      <div className="output">
        <div className="output-header">
          <span>Helius Activity (Lock PDA)</span>
        </div>
        {!heliusEnabled && (
          <div className="muted">
            Helius API key not configured. Add `VITE_HELIUS_API_KEY` or use a
            Helius RPC URL with `api-key`.
          </div>
        )}
        {heliusEnabled && heliusError && (
          <div className="alert error">{heliusError}</div>
        )}
        {heliusEnabled && !heliusError && heliusTxs.length === 0 && (
          <div className="muted">No Helius history yet.</div>
        )}
        {heliusEnabled && heliusTxs.length > 0 && (
          <ul className="status-list">
            {heliusTxs.map((tx) => (
              <li key={tx.signature}>
                {tx.type ?? 'Transaction'} · {tx.source ?? 'Helius'}{' '}
                <a
                  className="muted"
                  href={explorerUrl('tx', tx.signature)}
                  target="_blank"
                  rel="noreferrer"
                >
                  View tx
                </a>
                {tx.timestamp && (
                  <span className="muted">
                    {' '}
                    {new Date(tx.timestamp * 1000).toLocaleString()}
                  </span>
                )}
                {tx.description ? (
                  <div className="muted">{tx.description}</div>
                ) : null}
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  )
}

import { AnchorProvider, BN, Program } from '@coral-xyz/anchor'
import { type WalletContextState } from '@solana/wallet-adapter-react'
import {
  Connection,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from '@solana/web3.js'
import {
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddress,
} from '@solana/spl-token'
import { ATOMIC_LOCK_IDL, ATOMIC_LOCK_PROGRAM_ID } from '../idl/atomic_lock'
import { hexToBytes } from './hex'

export const LOCK_SEED_PREFIX = new TextEncoder().encode('lock')
export const VAULT_SEED_PREFIX = new TextEncoder().encode('vault')

export function getProgram(
  connection: Connection,
  wallet: WalletContextState,
  programId?: PublicKey,
) {
  if (!wallet.publicKey) {
    throw new Error('Wallet not connected')
  }
  const provider = new AnchorProvider(connection, wallet as never, {
    preflightCommitment: 'confirmed',
  })
  const id = programId ?? ATOMIC_LOCK_PROGRAM_ID
  return new Program(ATOMIC_LOCK_IDL, id, provider)
}

export function parseHex32(value: string): Uint8Array {
  const bytes = hexToBytes(value)
  if (bytes.length !== 32) {
    throw new Error('Expected 32-byte hex string')
  }
  return bytes
}

export function parseU64(value: string): BN {
  if (!value.trim()) {
    throw new Error('Amount is required')
  }
  const parsed = BigInt(value)
  if (parsed < 0n) {
    throw new Error('Amount must be positive')
  }
  return new BN(parsed.toString())
}

export function parseI64(value: string): BN {
  if (!value.trim()) {
    throw new Error('Lock time is required')
  }
  const parsed = BigInt(value)
  return new BN(parsed.toString())
}

export function deriveLockPda(
  depositor: PublicKey,
  hashlock: Uint8Array,
  programId: PublicKey,
) {
  return PublicKey.findProgramAddressSync(
    [LOCK_SEED_PREFIX, depositor.toBuffer(), Buffer.from(hashlock)],
    programId,
  )
}

export function deriveVaultPda(lockPda: PublicKey, programId: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [VAULT_SEED_PREFIX, lockPda.toBuffer()],
    programId,
  )
}

export async function ensureAssociatedTokenAccount(
  connection: Connection,
  owner: PublicKey,
  mint: PublicKey,
  payer: PublicKey,
) {
  const ata = await getAssociatedTokenAddress(mint, owner, false)
  const info = await connection.getAccountInfo(ata)
  if (info) {
    return { ata, ix: null }
  }
  const ix = createAssociatedTokenAccountInstruction(payer, ata, owner, mint)
  return { ata, ix }
}

export async function fetchPriorityFeeEstimate(
  rpcUrl: string,
  accountKeys: PublicKey[],
  connection?: Connection,
): Promise<{ estimate: number | null; source: 'helius' | 'fallback' | null }> {
  const priorityLevel =
    (import.meta.env.VITE_HELIUS_PRIORITY_LEVEL as string | undefined) ??
    'Medium'
  const priorityRpcUrl =
    (import.meta.env.VITE_HELIUS_PRIORITY_RPC_URL as string | undefined) ??
    rpcUrl
  const isHelius = priorityRpcUrl.toLowerCase().includes('helius')

  const payload = {
    jsonrpc: '2.0',
    id: 'priority-fee',
    method: 'getPriorityFeeEstimate',
    params: [
      {
        accountKeys: accountKeys.map((key) => key.toBase58()),
        options: {
          priorityLevel,
          recommended: true,
        },
      },
    ],
  }

  if (isHelius) {
    try {
      const response = await fetch(priorityRpcUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      })
      const json = await response.json()
      const estimate = json?.result?.priorityFeeEstimate
      if (typeof estimate === 'number') {
        return { estimate, source: 'helius' }
      }
    } catch {
      // fall through to fallback
    }
  }

  if (connection) {
    try {
      const fees = await (connection as Connection & {
        getRecentPrioritizationFees?: (
          accounts?: PublicKey[],
        ) => Promise<Array<{ prioritizationFee: number }>>
      }).getRecentPrioritizationFees?.(accountKeys)
      const values = (fees ?? [])
        .map((entry) => entry.prioritizationFee)
        .filter((value) => typeof value === 'number' && value > 0)
        .sort((a, b) => a - b)
      if (values.length > 0) {
        const mid = Math.floor(values.length / 2)
        const estimate =
          values.length % 2 === 0
            ? Math.round((values[mid - 1] + values[mid]) / 2)
            : values[mid]
        return { estimate, source: 'fallback' }
      }
    } catch {
      // ignore fallback failures
    }
  }
  return { estimate: null, source: null }
}

export const SYSTEM_PROGRAM = SystemProgram.programId
export const RENT_SYSVAR = SYSVAR_RENT_PUBKEY
export const TOKEN_PROGRAM = TOKEN_PROGRAM_ID

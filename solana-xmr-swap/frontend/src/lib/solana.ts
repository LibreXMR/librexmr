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

export const SYSTEM_PROGRAM = SystemProgram.programId
export const RENT_SYSVAR = SYSVAR_RENT_PUBKEY
export const TOKEN_PROGRAM = TOKEN_PROGRAM_ID

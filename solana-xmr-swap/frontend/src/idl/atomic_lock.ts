import type { Idl } from '@coral-xyz/anchor'
import { PublicKey } from '@solana/web3.js'

export const ATOMIC_LOCK_PROGRAM_ID = new PublicKey(
  'GpE4KVAv1Pzc2uhACFU6dAcLac46995Ueh2bhrKngtrM',
)

export const ATOMIC_LOCK_IDL = {
  address: ATOMIC_LOCK_PROGRAM_ID.toBase58(),
  metadata: {
    name: 'atomic_lock',
    version: '0.1.0',
    spec: '0.1.0',
  },
  instructions: [
    {
      name: 'initialize',
      accounts: [
        { name: 'depositor', isMut: true, isSigner: true },
        { name: 'atomic_lock', isMut: true, isSigner: false },
        { name: 'vault', isMut: true, isSigner: false },
        { name: 'depositor_token', isMut: true, isSigner: false },
        { name: 'token_mint', isMut: false, isSigner: false },
        { name: 'token_program', isMut: false, isSigner: false },
        { name: 'system_program', isMut: false, isSigner: false },
        { name: 'rent', isMut: false, isSigner: false },
      ],
      args: [
        { name: 'hashlock', type: { array: ['u8', 32] } },
        { name: 'adaptor_point', type: { array: ['u8', 32] } },
        { name: 'second_point', type: { array: ['u8', 32] } },
        { name: 'y_point', type: { array: ['u8', 32] } },
        { name: 'r1', type: { array: ['u8', 32] } },
        { name: 'r2', type: { array: ['u8', 32] } },
        { name: 'challenge', type: { array: ['u8', 32] } },
        { name: 'response', type: { array: ['u8', 32] } },
        { name: 'lock_until', type: 'i64' },
        { name: 'amount', type: 'u64' },
      ],
    },
    {
      name: 'verify_dleq',
      accounts: [{ name: 'atomic_lock', isMut: true, isSigner: false }],
      args: [],
    },
    {
      name: 'verify_and_unlock',
      accounts: [
        { name: 'unlocker', isMut: true, isSigner: true },
        { name: 'atomic_lock', isMut: true, isSigner: false },
        { name: 'vault', isMut: true, isSigner: false },
        { name: 'unlocker_token', isMut: true, isSigner: false },
        { name: 'token_program', isMut: false, isSigner: false },
      ],
      args: [{ name: 'secret', type: { array: ['u8', 32] } }],
    },
    {
      name: 'refund',
      accounts: [
        { name: 'depositor', isMut: true, isSigner: true },
        { name: 'atomic_lock', isMut: true, isSigner: false },
        { name: 'vault', isMut: true, isSigner: false },
        { name: 'depositor_token', isMut: true, isSigner: false },
        { name: 'token_program', isMut: false, isSigner: false },
      ],
      args: [],
    },
  ],
  accounts: [
    {
      name: 'AtomicLock',
      type: {
        kind: 'struct',
        fields: [
          { name: 'depositor', type: 'publicKey' },
          { name: 'hashlock', type: { array: ['u8', 32] } },
          { name: 'adaptor_point', type: { array: ['u8', 32] } },
          { name: 'second_point', type: { array: ['u8', 32] } },
          { name: 'y_point', type: { array: ['u8', 32] } },
          { name: 'r1', type: { array: ['u8', 32] } },
          { name: 'r2', type: { array: ['u8', 32] } },
          { name: 'challenge', type: { array: ['u8', 32] } },
          { name: 'response', type: { array: ['u8', 32] } },
          { name: 'lock_until', type: 'i64' },
          { name: 'amount', type: 'u64' },
          { name: 'token_mint', type: 'publicKey' },
          { name: 'vault', type: 'publicKey' },
          { name: 'dleq_verified', type: 'bool' },
          { name: 'unlocked', type: 'bool' },
          { name: 'lock_bump', type: 'u8' },
        ],
      },
    },
  ],
} as unknown as Idl

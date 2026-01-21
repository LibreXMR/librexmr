import { describe, expect, it } from 'vitest'
import {
  validateHexBytes,
  validateI64,
  validatePublicKey,
  validateU64,
  validateUrl,
} from './validation'

describe('validation helpers', () => {
  it('validates public keys', () => {
    expect(validatePublicKey('', 'Program ID')).toBe('Program ID is required')
    expect(validatePublicKey('bad', 'Program ID')).toBe('Program ID is invalid')
    expect(
      validatePublicKey('11111111111111111111111111111111', 'Program ID'),
    ).toBeNull()
  })

  it('validates hex byte strings', () => {
    expect(validateHexBytes('', 32, 'Hashlock')).toBe('Hashlock is required')
    expect(validateHexBytes('zz', 32, 'Hashlock')).toBe('Hashlock must be hex')
    expect(validateHexBytes('aa', 32, 'Hashlock')).toBe(
      'Hashlock must be 64 hex chars',
    )
    expect(validateHexBytes('aa'.repeat(32), 32, 'Hashlock')).toBeNull()
  })

  it('validates numeric inputs', () => {
    expect(validateU64('', 'Amount')).toBe('Amount is required')
    expect(validateU64('-1', 'Amount')).toBe('Amount must be positive')
    expect(validateU64('nope', 'Amount')).toBe('Amount must be a number')
    expect(validateU64('10', 'Amount')).toBeNull()

    expect(validateI64('', 'Lock time')).toBe('Lock time is required')
    expect(validateI64('nope', 'Lock time')).toBe('Lock time must be a number')
    expect(validateI64('1700000000', 'Lock time')).toBeNull()
  })

  it('validates URLs', () => {
    expect(validateUrl('', 'RPC URL')).toBe('RPC URL is required')
    expect(validateUrl('not-a-url', 'RPC URL')).toBe(
      'RPC URL must be a valid URL',
    )
    expect(validateUrl('https://api.devnet.solana.com', 'RPC URL')).toBeNull()
  })
})

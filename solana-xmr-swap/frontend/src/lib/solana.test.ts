import { describe, expect, it } from 'vitest'
import { parseHex32, parseI64, parseU64 } from './solana'

describe('solana helpers', () => {
  it('parses 32-byte hex', () => {
    const value = 'aa'.repeat(32)
    const out = parseHex32(value)
    expect(out).toHaveLength(32)
  })

  it('rejects invalid hex length', () => {
    expect(() => parseHex32('aa')).toThrow()
  })

  it('parses numeric fields', () => {
    expect(parseU64('10').toString()).toBe('10')
    expect(parseI64('1700000000').toString()).toBe('1700000000')
  })
})

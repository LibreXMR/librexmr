import { describe, expect, it } from 'vitest'
import { sampleAuditLog } from '../data/samples'
import { verifyAuditLog } from './audit'

describe('verifyAuditLog', () => {
  it('accepts the signed sample audit log', () => {
    const result = verifyAuditLog(sampleAuditLog)
    expect(result.hashMatches).toBe(true)
    expect(result.signatureValid).toBe(true)
  })

  it('rejects when transcript changes', () => {
    const tampered = {
      ...sampleAuditLog,
      payload: {
        ...sampleAuditLog.payload,
        transcript: {
          ...sampleAuditLog.payload.transcript,
          hashlock: '00'.repeat(32),
        },
      },
    }
    const result = verifyAuditLog(tampered)
    expect(result.hashMatches).toBe(false)
    expect(result.signatureValid).toBe(false)
  })
})

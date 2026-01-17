import { describe, expect, it } from 'vitest'
import { sampleAlert } from '../data/samples'
import { verifyAlertEnvelope } from './alert'

describe('verifyAlertEnvelope', () => {
  it('accepts the signed sample alert', () => {
    const result = verifyAlertEnvelope(sampleAlert)
    expect(result.hashMatches).toBe(true)
    expect(result.signatureValid).toBe(true)
  })

  it('rejects when payload changes', () => {
    const tampered = { ...sampleAlert, expired: true }
    const result = verifyAlertEnvelope(tampered)
    expect(result.hashMatches).toBe(false)
    expect(result.signatureValid).toBe(false)
  })
})

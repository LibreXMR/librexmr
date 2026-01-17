import { describe, expect, it } from 'vitest'
import { demoSwap } from '../data/samples'
import { verifyDleqClientSide } from './dleq'

describe('verifyDleqClientSide', () => {
  it('verifies the demo swap vector', () => {
    const result = verifyDleqClientSide({
      adaptorPoint: demoSwap.adaptorPoint,
      secondPoint: demoSwap.secondPoint,
      yPoint: demoSwap.yPoint,
      r1: demoSwap.r1,
      r2: demoSwap.r2,
      challenge: demoSwap.challenge,
      response: demoSwap.response,
      hashlock: demoSwap.hashlock,
    })
    expect(result.ok).toBe(true)
    expect(result.report.challengeMatches).toBe(true)
  })

  it('fails when the challenge is wrong', () => {
    const result = verifyDleqClientSide({
      adaptorPoint: demoSwap.adaptorPoint,
      secondPoint: demoSwap.secondPoint,
      yPoint: demoSwap.yPoint,
      r1: demoSwap.r1,
      r2: demoSwap.r2,
      challenge: '00'.repeat(32),
      response: demoSwap.response,
      hashlock: demoSwap.hashlock,
    })
    expect(result.ok).toBe(false)
    expect(result.report.challengeMatches).toBe(false)
  })
})

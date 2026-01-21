import { describe, expect, it } from 'vitest'
import { retryAsync } from './retry'

describe('retryAsync', () => {
  it('retries until success', async () => {
    let calls = 0
    const result = await retryAsync(
      async () => {
        calls += 1
        if (calls < 3) {
          throw new Error('fail')
        }
        return 42
      },
      {
        retries: 3,
        baseDelayMs: 1,
        maxDelayMs: 2,
        jitterMs: 0,
      },
    )
    expect(result).toBe(42)
    expect(calls).toBe(3)
  })

  it('throws after retries', async () => {
    let calls = 0
    await expect(
      retryAsync(
        async () => {
          calls += 1
          throw new Error('fail')
        },
        {
          retries: 1,
          baseDelayMs: 1,
          maxDelayMs: 2,
          jitterMs: 0,
        },
      ),
    ).rejects.toThrow('fail')
    expect(calls).toBe(2)
  })
})

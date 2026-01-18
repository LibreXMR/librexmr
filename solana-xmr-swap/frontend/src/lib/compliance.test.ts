import { describe, expect, it, vi, afterEach } from 'vitest'
import { screenAddress } from './compliance'

afterEach(() => {
  vi.unstubAllEnvs()
  vi.unstubAllGlobals()
})

describe('compliance screening', () => {
  it('skips when api key not set', async () => {
    const fetchMock = vi.fn()
    vi.stubGlobal('fetch', fetchMock)
    const result = await screenAddress('test')
    expect(result.source).toBe('skipped')
    expect(result.isClean).toBe(true)
    expect(fetchMock).not.toHaveBeenCalled()
  })

  it('calls range api when configured', async () => {
    vi.stubEnv('VITE_RANGE_API_KEY', 'test-key')
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        isClean: true,
        riskScore: 0,
      }),
    })
    vi.stubGlobal('fetch', fetchMock)
    const result = await screenAddress('wallet')
    expect(result.source).toBe('range')
    expect(fetchMock).toHaveBeenCalledTimes(1)
  })

  it('throws on non-ok response', async () => {
    vi.stubEnv('VITE_RANGE_API_KEY', 'test-key')
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      status: 401,
    })
    vi.stubGlobal('fetch', fetchMock)
    await expect(screenAddress('wallet')).rejects.toThrow(
      'Compliance screening failed: 401',
    )
  })

  it('fails closed when configured without api key', async () => {
    vi.stubEnv('VITE_RANGE_FAIL_CLOSED', 'true')
    await expect(screenAddress('wallet')).rejects.toThrow(
      'Compliance API key required',
    )
  })
})

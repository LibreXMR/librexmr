import { describe, expect, it, vi, afterEach } from 'vitest'
import {
  fetchHeliusTransactions,
  getHeliusApiConfig,
  isHeliusEnabled,
  parseApiKeyFromRpc,
} from './helius'

afterEach(() => {
  vi.unstubAllEnvs()
  vi.unstubAllGlobals()
})

describe('helius helpers', () => {
  it('parses api key from rpc url', () => {
    expect(
      parseApiKeyFromRpc('https://devnet.helius-rpc.com/?api-key=abc123'),
    ).toBe('abc123')
  })

  it('detects enabled status from env', () => {
    vi.stubEnv('VITE_HELIUS_API_KEY', 'env-key')
    expect(isHeliusEnabled('https://api.devnet.solana.com')).toBe(true)
  })

  it('builds api config from rpc url', () => {
    const config = getHeliusApiConfig(
      'https://devnet.helius-rpc.com/?api-key=rpc-key',
    )
    expect(config?.apiKey).toBe('rpc-key')
  })

  it('returns empty history when helius not configured', async () => {
    const fetchMock = vi.fn()
    vi.stubGlobal('fetch', fetchMock)
    const data = await fetchHeliusTransactions(
      'https://api.devnet.solana.com',
      'address',
      10,
    )
    expect(data).toEqual([])
    expect(fetchMock).not.toHaveBeenCalled()
  })

  it('caps the history limit and calls helius', async () => {
    vi.stubEnv('VITE_HELIUS_API_KEY', 'env-key')
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => [{ signature: 'sig', timestamp: 0 }],
    })
    vi.stubGlobal('fetch', fetchMock)
    const data = await fetchHeliusTransactions(
      'https://api.devnet.solana.com',
      'address',
      250,
    )
    expect(data).toHaveLength(1)
    const url = fetchMock.mock.calls[0][0] as string
    expect(url).toContain('limit=50')
  })

  it('throws on non-ok helius responses', async () => {
    vi.stubEnv('VITE_HELIUS_API_KEY', 'env-key')
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      status: 500,
      json: async () => ({}),
    })
    vi.stubGlobal('fetch', fetchMock)
    await expect(
      fetchHeliusTransactions(
        'https://api.devnet.solana.com',
        'address',
        10,
      ),
    ).rejects.toThrow('Helius API error: 500')
  })
})

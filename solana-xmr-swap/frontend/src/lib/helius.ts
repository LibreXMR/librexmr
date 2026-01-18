export type HeliusTransaction = {
  signature: string
  timestamp: number
  description?: string
  type?: string
  source?: string
}

type HeliusApiConfig = {
  apiKey: string
  apiUrl: string
}

const DEFAULT_API_URL = 'https://api.helius.xyz'
const DEFAULT_TIMEOUT_MS = 6000
const MAX_LIMIT = 50

export function parseApiKeyFromRpc(rpcUrl: string): string | null {
  try {
    const url = new URL(rpcUrl)
    return url.searchParams.get('api-key')
  } catch {
    return null
  }
}

export function getHeliusApiConfig(rpcUrl: string): HeliusApiConfig | null {
  const apiKey =
    (import.meta.env.VITE_HELIUS_API_KEY as string | undefined) ??
    parseApiKeyFromRpc(rpcUrl)
  if (!apiKey) {
    return null
  }
  const apiUrl =
    (import.meta.env.VITE_HELIUS_API_URL as string | undefined) ?? DEFAULT_API_URL
  return { apiKey, apiUrl }
}

export function isHeliusEnabled(rpcUrl: string): boolean {
  return getHeliusApiConfig(rpcUrl) !== null
}

export async function fetchHeliusTransactions(
  rpcUrl: string,
  address: string,
  limit = 10,
): Promise<HeliusTransaction[]> {
  const config = getHeliusApiConfig(rpcUrl)
  if (!config) {
    return []
  }
  if (!address) {
    return []
  }
  const boundedLimit = Math.max(1, Math.min(limit, MAX_LIMIT))
  const timeoutMs = Number(
    (import.meta.env.VITE_HELIUS_TIMEOUT_MS as string | undefined) ??
      DEFAULT_TIMEOUT_MS,
  )
  const url = `${config.apiUrl}/v0/addresses/${address}/transactions?api-key=${config.apiKey}&limit=${boundedLimit}`
  const controller = new AbortController()
  const timer = window.setTimeout(() => controller.abort(), timeoutMs)
  const response = await fetch(url, { signal: controller.signal })
  window.clearTimeout(timer)
  if (!response.ok) {
    throw new Error(`Helius API error: ${response.status}`)
  }
  const data = (await response.json()) as HeliusTransaction[]
  return data
}

export type ScreeningResult = {
  address: string
  isClean: boolean
  riskScore: number
  timestamp: number
  source: 'range' | 'skipped'
}

const DEFAULT_RANGE_URL = 'https://api.range.org'

function isFailClosed(): boolean {
  const value = (import.meta.env.VITE_RANGE_FAIL_CLOSED as string | undefined) ?? ''
  return value.toLowerCase() === 'true'
}

function getRangeConfig() {
  const apiKey = import.meta.env.VITE_RANGE_API_KEY as string | undefined
  if (!apiKey) {
    if (isFailClosed()) {
      throw new Error('Compliance API key required')
    }
    return null
  }
  const apiUrl =
    (import.meta.env.VITE_RANGE_API_URL as string | undefined) ?? DEFAULT_RANGE_URL
  return { apiKey, apiUrl }
}

export async function screenAddress(address: string): Promise<ScreeningResult> {
  const config = getRangeConfig()
  if (!config) {
    return {
      address,
      isClean: true,
      riskScore: 0,
      timestamp: Date.now(),
      source: 'skipped',
    }
  }

  const response = await fetch(`${config.apiUrl}/v1/screen`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${config.apiKey}`,
    },
    body: JSON.stringify({ address, chain: 'solana' }),
  })

  if (!response.ok) {
    throw new Error(`Compliance screening failed: ${response.status}`)
  }

  const data = (await response.json()) as {
    isClean: boolean
    riskScore: number
    timestamp?: number
  }

  return {
    address,
    isClean: data.isClean,
    riskScore: data.riskScore,
    timestamp: data.timestamp ?? Date.now(),
    source: 'range',
  }
}

export async function screenSwapParties(
  depositor: string,
  recipient?: string,
): Promise<{ depositor: ScreeningResult; recipient?: ScreeningResult }> {
  const [depositorResult, recipientResult] = await Promise.all([
    screenAddress(depositor),
    recipient ? screenAddress(recipient) : Promise.resolve(undefined),
  ])

  return { depositor: depositorResult, recipient: recipientResult }
}

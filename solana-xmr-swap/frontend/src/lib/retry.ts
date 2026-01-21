export type RetryOptions = {
  retries: number
  baseDelayMs: number
  maxDelayMs: number
  jitterMs: number
}

const DEFAULT_RETRY: RetryOptions = {
  retries: 2,
  baseDelayMs: 200,
  maxDelayMs: 2000,
  jitterMs: 200,
}

export function getRetryOptions(): RetryOptions {
  const read = (key: string, fallback: number) => {
    const value = (import.meta.env[key] as string | undefined) ?? ''
    const parsed = Number(value)
    return Number.isFinite(parsed) ? parsed : fallback
  }
  return {
    retries: read('VITE_RPC_RETRY_MAX', DEFAULT_RETRY.retries),
    baseDelayMs: read('VITE_RPC_RETRY_BASE_MS', DEFAULT_RETRY.baseDelayMs),
    maxDelayMs: read('VITE_RPC_RETRY_MAX_MS', DEFAULT_RETRY.maxDelayMs),
    jitterMs: read('VITE_RPC_RETRY_JITTER_MS', DEFAULT_RETRY.jitterMs),
  }
}

const sleep = (ms: number) =>
  new Promise<void>((resolve) => {
    window.setTimeout(resolve, ms)
  })

export async function retryAsync<T>(
  action: () => Promise<T>,
  options: RetryOptions = DEFAULT_RETRY,
): Promise<T> {
  let lastError: unknown
  for (let attempt = 0; attempt <= options.retries; attempt += 1) {
    try {
      return await action()
    } catch (err) {
      lastError = err
      if (attempt >= options.retries) {
        throw err
      }
      const backoff = Math.min(
        options.baseDelayMs * 2 ** attempt,
        options.maxDelayMs,
      )
      const jitter =
        options.jitterMs > 0
          ? Math.floor(Math.random() * (options.jitterMs + 1))
          : 0
      await sleep(backoff + jitter)
    }
  }
  throw lastError
}

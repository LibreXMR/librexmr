import { PublicKey } from '@solana/web3.js'

export function validatePublicKey(
  value: string,
  label: string,
  required = true,
): string | null {
  const trimmed = value.trim()
  if (!trimmed) {
    return required ? `${label} is required` : null
  }
  try {
    new PublicKey(trimmed)
    return null
  } catch {
    return `${label} is invalid`
  }
}

export function validateHexBytes(
  value: string,
  bytes: number,
  label: string,
  required = true,
): string | null {
  const trimmed = value.trim()
  if (!trimmed) {
    return required ? `${label} is required` : null
  }
  if (!/^[0-9a-fA-F]+$/.test(trimmed)) {
    return `${label} must be hex`
  }
  if (trimmed.length !== bytes * 2) {
    return `${label} must be ${bytes * 2} hex chars`
  }
  return null
}

export function validateU64(value: string, label: string): string | null {
  const trimmed = value.trim()
  if (!trimmed) {
    return `${label} is required`
  }
  try {
    const parsed = BigInt(trimmed)
    if (parsed < 0n) {
      return `${label} must be positive`
    }
    return null
  } catch {
    return `${label} must be a number`
  }
}

export function validateI64(value: string, label: string): string | null {
  const trimmed = value.trim()
  if (!trimmed) {
    return `${label} is required`
  }
  try {
    BigInt(trimmed)
    return null
  } catch {
    return `${label} must be a number`
  }
}

export function validateUrl(value: string, label: string): string | null {
  const trimmed = value.trim()
  if (!trimmed) {
    return `${label} is required`
  }
  try {
    new URL(trimmed)
    return null
  } catch {
    return `${label} must be a valid URL`
  }
}

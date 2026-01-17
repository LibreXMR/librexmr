export function hexToBytes(hex: string): Uint8Array {
  const normalized = hex.trim().toLowerCase()
  if (normalized.length % 2 !== 0) {
    throw new Error('Hex string has odd length')
  }
  const out = new Uint8Array(normalized.length / 2)
  for (let i = 0; i < normalized.length; i += 2) {
    const byte = Number.parseInt(normalized.slice(i, i + 2), 16)
    if (Number.isNaN(byte)) {
      throw new Error('Invalid hex string')
    }
    out[i / 2] = byte
  }
  return out
}

export function bytesToHex(bytes: Uint8Array): string {
  const hex: string[] = []
  for (const byte of bytes) {
    hex.push(byte.toString(16).padStart(2, '0'))
  }
  return hex.join('')
}

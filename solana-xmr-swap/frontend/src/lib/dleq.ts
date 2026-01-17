import { blake2s } from '@noble/hashes/blake2.js'
import { ed25519 } from '@noble/curves/ed25519.js'
import { bytesToHex, hexToBytes } from './hex'

const DLEQ_DOMAIN_TAG = new TextEncoder().encode('DLEQ')
const ED25519_BASEPOINT_COMPRESSED = hexToBytes(
  '5866666666666666666666666666666666666666666666666666666666666666',
)
const ED25519_GROUP_ORDER = ed25519.Point.CURVE().n

export type DleqParams = {
  adaptorPoint: string
  secondPoint: string
  yPoint: string
  r1: string
  r2: string
  challenge: string
  response: string
  hashlock: string
}

export type DleqReport = {
  computedChallenge: string
  challengeMatches: boolean
  lhsR1Matches: boolean
  lhsR2Matches: boolean
}

export type DleqVerification = {
  ok: boolean
  report: DleqReport
}

export function verifyDleqClientSide(params: DleqParams): DleqVerification {
  const adaptorPoint = decodeHex32(params.adaptorPoint, 'adaptor_point')
  const secondPoint = decodeHex32(params.secondPoint, 'second_point')
  const yPoint = decodeHex32(params.yPoint, 'y_point')
  const r1 = decodeHex32(params.r1, 'r1')
  const r2 = decodeHex32(params.r2, 'r2')
  const challenge = decodeHex32(params.challenge, 'challenge')
  const response = decodeHex32(params.response, 'response')
  const hashlock = decodeHex32(params.hashlock, 'hashlock')

  const computedChallenge = computeChallengeBytes(
    yPoint,
    adaptorPoint,
    secondPoint,
    r1,
    r2,
    hashlock,
  )

  const challengeMatches = bytesEqual(computedChallenge, challenge)
  if (!challengeMatches) {
    return {
      ok: false,
      report: {
        computedChallenge: bytesToHex(computedChallenge),
        challengeMatches,
        lhsR1Matches: false,
        lhsR2Matches: false,
      },
    }
  }

  const tPoint = decodePoint(adaptorPoint, 'adaptor_point')
  const uPoint = decodePoint(secondPoint, 'second_point')
  const yPointDecoded = decodePoint(yPoint, 'y_point')
  const r1Point = decodePoint(r1, 'r1')
  const r2Point = decodePoint(r2, 'r2')

  ensureNotSmallOrder(tPoint, 'adaptor_point')
  ensureNotSmallOrder(uPoint, 'second_point')
  ensureNotSmallOrder(yPointDecoded, 'y_point')
  ensureNotSmallOrder(r1Point, 'r1')
  ensureNotSmallOrder(r2Point, 'r2')

  const c = bytesToNumberLE(challenge)
  const s = bytesToNumberLE(response)

  const sG = ed25519.Point.BASE.multiply(s)
  const sY = yPointDecoded.multiply(s)
  const cT = tPoint.multiply(c)
  const cU = uPoint.multiply(c)

  const r1Check = r1Point.add(cT)
  const r2Check = r2Point.add(cU)

  const lhsR1Matches = sG.equals(r1Check)
  const lhsR2Matches = sY.equals(r2Check)

  return {
    ok: lhsR1Matches && lhsR2Matches,
    report: {
      computedChallenge: bytesToHex(computedChallenge),
      challengeMatches,
      lhsR1Matches,
      lhsR2Matches,
    },
  }
}

function computeChallengeBytes(
  yPoint: Uint8Array,
  adaptorPoint: Uint8Array,
  secondPoint: Uint8Array,
  r1: Uint8Array,
  r2: Uint8Array,
  hashlock: Uint8Array,
): Uint8Array {
  const hash = blake2s(
    concatBytes(
      DLEQ_DOMAIN_TAG,
      ED25519_BASEPOINT_COMPRESSED,
      yPoint,
      adaptorPoint,
      secondPoint,
      r1,
      r2,
      hashlock,
    ),
    { dkLen: 32 },
  )
  const reduced = mod(bytesToNumberLE(hash), ED25519_GROUP_ORDER)
  return numberToBytesLE(reduced, 32)
}

function decodePoint(bytes: Uint8Array, field: string) {
  try {
    return ed25519.Point.fromBytes(bytes)
  } catch {
    throw new Error(`invalid point encoding: ${field}`)
  }
}

function ensureNotSmallOrder(point: typeof ed25519.Point.BASE, field: string) {
  if (isSmallOrder(point)) {
    throw new Error(`small-order point: ${field}`)
  }
}

function isSmallOrder(point: typeof ed25519.Point.BASE): boolean {
  return point.isSmallOrder()
}

function decodeHex32(value: string, field: string): Uint8Array {
  const normalized = strip0x(value)
  const bytes = hexToBytes(normalized)
  if (bytes.length !== 32) {
    throw new Error(`invalid length for ${field}: expected 32, got ${bytes.length}`)
  }
  return bytes
}

function strip0x(value: string): string {
  return value.trim().toLowerCase().startsWith('0x') ? value.trim().slice(2) : value.trim()
}

function concatBytes(...chunks: Uint8Array[]): Uint8Array {
  const total = chunks.reduce((sum, chunk) => sum + chunk.length, 0)
  const out = new Uint8Array(total)
  let offset = 0
  for (const chunk of chunks) {
    out.set(chunk, offset)
    offset += chunk.length
  }
  return out
}

function bytesToNumberLE(bytes: Uint8Array): bigint {
  let value = 0n
  for (let i = bytes.length - 1; i >= 0; i -= 1) {
    value = (value << 8n) + BigInt(bytes[i])
  }
  return value
}

function numberToBytesLE(value: bigint, length: number): Uint8Array {
  const out = new Uint8Array(length)
  let current = value
  for (let i = 0; i < length; i += 1) {
    out[i] = Number(current & 0xffn)
    current >>= 8n
  }
  return out
}

function mod(value: bigint, modulo: bigint): bigint {
  const result = value % modulo
  return result >= 0n ? result : result + modulo
}

function bytesEqual(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) return false
  for (let i = 0; i < a.length; i += 1) {
    if (a[i] !== b[i]) return false
  }
  return true
}

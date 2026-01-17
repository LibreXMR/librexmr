import { sha256Hex, verifyEd25519 } from './crypto'

export type AlertPayload = {
  event: string
  pda: string
  dleq_verified: boolean
  unlocked: boolean
  expired: boolean
  now_unix: number
  lock_until: number
}

export type AlertEnvelope = AlertPayload & {
  payload_hash?: string
  signature?: string
  public_key?: string
}

export type AlertVerification = {
  payloadHash: string
  hashMatches: boolean
  signaturePresent: boolean
  signatureValid: boolean
  error?: string
}

export function buildAlertPayload(envelope: AlertEnvelope): AlertPayload {
  return {
    event: envelope.event,
    pda: envelope.pda,
    dleq_verified: envelope.dleq_verified,
    unlocked: envelope.unlocked,
    expired: envelope.expired,
    now_unix: envelope.now_unix,
    lock_until: envelope.lock_until,
  }
}

export function serializeAlertPayload(payload: AlertPayload): string {
  return [
    '{',
    `"event":${JSON.stringify(payload.event)}`,
    `,"pda":${JSON.stringify(payload.pda)}`,
    `,"dleq_verified":${payload.dleq_verified}`,
    `,"unlocked":${payload.unlocked}`,
    `,"expired":${payload.expired}`,
    `,"now_unix":${payload.now_unix}`,
    `,"lock_until":${payload.lock_until}`,
    '}',
  ].join('')
}

export function alertPayloadBytes(payload: AlertPayload): Uint8Array {
  return new TextEncoder().encode(serializeAlertPayload(payload))
}

export function verifyAlertEnvelope(envelope: AlertEnvelope): AlertVerification {
  const payload = buildAlertPayload(envelope)
  const payloadBytes = alertPayloadBytes(payload)
  const payloadHash = sha256Hex(payloadBytes)
  const hashMatches = envelope.payload_hash
    ? envelope.payload_hash === payloadHash
    : false
  const signaturePresent = Boolean(envelope.signature && envelope.public_key)

  let signatureValid = false
  let error: string | undefined

  if (signaturePresent) {
    signatureValid = verifyEd25519(
      payloadBytes,
      envelope.signature ?? '',
      envelope.public_key ?? '',
    )
  } else if (envelope.signature || envelope.public_key) {
    error = 'Signature or public key missing'
  }

  return {
    payloadHash,
    hashMatches,
    signaturePresent,
    signatureValid,
    error,
  }
}

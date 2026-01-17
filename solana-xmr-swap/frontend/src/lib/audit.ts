import { sha256Hex, verifyEd25519 } from './crypto'

export type AuditReport = {
  computed_challenge: string
  challenge_matches: boolean
  lhs_r1_matches: boolean
  lhs_r2_matches: boolean
}

export type AuditTranscript = {
  adaptor_point: string
  second_point: string
  y_point: string
  r1: string
  r2: string
  challenge: string
  response: string
  hashlock: string
}

export type AuditPayload = {
  timestamp_unix: number
  input_path: string
  ok: boolean
  report: AuditReport
  transcript: AuditTranscript
}

export type SignedAuditLog = {
  payload: AuditPayload
  payload_hash: string
  signature?: string
  public_key?: string
}

export type AuditVerification = {
  payloadHash: string
  hashMatches: boolean
  signaturePresent: boolean
  signatureValid: boolean
  error?: string
}

export function serializeAuditPayload(payload: AuditPayload): string {
  const report = payload.report
  const transcript = payload.transcript
  return [
    '{',
    `"timestamp_unix":${payload.timestamp_unix}`,
    `,"input_path":${JSON.stringify(payload.input_path)}`,
    `,"ok":${payload.ok}`,
    ',"report":{',
    `"computed_challenge":${JSON.stringify(report.computed_challenge)}`,
    `,"challenge_matches":${report.challenge_matches}`,
    `,"lhs_r1_matches":${report.lhs_r1_matches}`,
    `,"lhs_r2_matches":${report.lhs_r2_matches}`,
    '}',
    ',"transcript":{',
    `"adaptor_point":${JSON.stringify(transcript.adaptor_point)}`,
    `,"second_point":${JSON.stringify(transcript.second_point)}`,
    `,"y_point":${JSON.stringify(transcript.y_point)}`,
    `,"r1":${JSON.stringify(transcript.r1)}`,
    `,"r2":${JSON.stringify(transcript.r2)}`,
    `,"challenge":${JSON.stringify(transcript.challenge)}`,
    `,"response":${JSON.stringify(transcript.response)}`,
    `,"hashlock":${JSON.stringify(transcript.hashlock)}`,
    '}',
    '}',
  ].join('')
}

export function auditPayloadBytes(payload: AuditPayload): Uint8Array {
  return new TextEncoder().encode(serializeAuditPayload(payload))
}

export function verifyAuditLog(log: SignedAuditLog): AuditVerification {
  const payloadBytes = auditPayloadBytes(log.payload)
  const payloadHash = sha256Hex(payloadBytes)
  const hashMatches = log.payload_hash === payloadHash
  const signaturePresent = Boolean(log.signature && log.public_key)

  let signatureValid = false
  let error: string | undefined

  if (signaturePresent) {
    signatureValid = verifyEd25519(
      payloadBytes,
      log.signature ?? '',
      log.public_key ?? '',
    )
  } else if (log.signature || log.public_key) {
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

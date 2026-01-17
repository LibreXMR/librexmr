import { sha256 } from '@noble/hashes/sha2.js'
import nacl from 'tweetnacl'
import { bytesToHex, hexToBytes } from './hex'

export function sha256Hex(data: Uint8Array): string {
  return bytesToHex(sha256(data))
}

export function verifyEd25519(
  message: Uint8Array,
  signatureHex: string,
  publicKeyHex: string,
): boolean {
  const signature = Uint8Array.from(hexToBytes(signatureHex))
  const publicKey = Uint8Array.from(hexToBytes(publicKeyHex))
  const msg = Uint8Array.from(message)
  if (signature.length !== nacl.sign.signatureLength) {
    return false
  }
  if (publicKey.length !== nacl.sign.publicKeyLength) {
    return false
  }
  return nacl.sign.detached.verify(msg, signature, publicKey)
}

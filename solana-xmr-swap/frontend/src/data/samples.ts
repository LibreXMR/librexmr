import type { AlertEnvelope } from '../lib/alert'
import type { SignedAuditLog } from '../lib/audit'

export const sampleAlert: AlertEnvelope = {
  event: 'state_change',
  pda: '11111111111111111111111111111111',
  dleq_verified: true,
  unlocked: false,
  expired: false,
  now_unix: 1710000000,
  lock_until: 1710003600,
  payload_hash:
    '95a9c30e852b77d181cc1b21c3a8906d9aabfd3a971aa468fc5dfbdc3db91424',
  signature:
    '19cab49a954a6edcb7a978958c03dc783d86075a8f66e2c11333121f3602c7940e58a2b465479cf0e0f4441036676b83583559b2d1baafaaa097468546f11908',
  public_key:
    'ea4a6c63e29c520abef5507b132ec5f9954776aebebe7b92421eea691446d22c',
}

export const sampleAuditLog: SignedAuditLog = {
  payload: {
    timestamp_unix: 1710000000,
    input_path: 'test_vectors/dleq.json',
    ok: true,
    report: {
      computed_challenge:
        'b2cd06dd3134e6e8b6fa532a1dd2c41ab963849e6c41ccc97b1fb425a163f00c',
      challenge_matches: true,
      lhs_r1_matches: true,
      lhs_r2_matches: true,
    },
    transcript: {
      adaptor_point:
        '85ce3cf603efcf45b599cce75369e854823864e471ad297d955f32db0ade7d42',
      second_point:
        'be7b5c4cf816760b7709df6b47b393d8cdd1605e06e2e2080944d684fad0795c',
      y_point:
        'c9a3f86aae465f0e56513864510f3997561fa2c9e85ea21dc2292309f3cd6022',
      r1: '34e31fa42ec011caed7fa1d72125b03ca52659e04a0e7aca42d9906f2509ef11',
      r2: '2ff0af08f9d4654db8e6cb72c226fbb42592f4f2df3f6f06cafbc033fd9884f5',
      challenge:
        'b2cd06dd3134e6e8b6fa532a1dd2c41ab963849e6c41ccc97b1fb425a163f00c',
      response:
        '1eedaa629d5bb28d173153ff275608169dc822d2c9dfb450af3254d9ff100802',
      hashlock:
        'b6acca81a0939a856c35e4c4188e95b91731aab1d4629a4cee79dd09ded4fc94',
    },
  },
  payload_hash:
    '47efbc9a3bf00d784589fb88932f2e61b4815530e5f3e2ae2a67027a7351208e',
  signature:
    'fe3ddd29950903f1127bf870b61d798abfc04e5b493769e15b8306ee93c2c83f3e3e96d4c7f644aa9793df87e460ed57e75df3379e46df097f0c994a619ec102',
  public_key:
    'ea4a6c63e29c520abef5507b132ec5f9954776aebebe7b92421eea691446d22c',
}

export const demoSwap = {
  hashlock: 'efd6fd6ffd37ef49e63fe7bff860dba068ef5f7e9e76e0b7aae7d6694839d420',
  secret: '7496f601de5dbd08c0f65d6927c4570f34a205c37bd98aa8ad83a91241937a0a',
  adaptorPoint:
    '253d3c600297c9d049fca09a284738a9c8839051b3fab91a5b7bf5a551be3cf4',
  secondPoint:
    '5d4d443ec3dd651b6f8de0f953e103a4a5d7ddd390952250694d99734d9aca6b',
  yPoint:
    'a833215ca65eb659f52be7d69c6797f0f753598b1af3c42f6dcab035919df6b7',
  r1: '3468df404ef63c188fa10e53e959500afe700c3567c544663c7670bf7326915a',
  r2: '03fedd2d8ae19d915007b5ba21c4e67f5181da2e6c2aa59953a2993bbaf2ac36',
  challenge:
    '1c9bb910f1aefb55ce8379ff42e8e3c030646d69574b48143f883c871f548f0b',
  response:
    '61a60f4d24d7cf19585d49aa4e3d00d9f2ceb61d5745c24e6efb3ab9fa9c370e',
}

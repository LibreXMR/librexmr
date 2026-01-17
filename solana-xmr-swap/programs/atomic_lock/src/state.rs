 use anchor_lang::prelude::*;
 
 #[account]
 pub struct AtomicLock {
     pub depositor: Pubkey,
     pub hashlock: [u8; 32],
     pub adaptor_point: [u8; 32],
     pub second_point: [u8; 32],
     pub y_point: [u8; 32],
     pub r1: [u8; 32],
     pub r2: [u8; 32],
     pub challenge: [u8; 32],
     pub response: [u8; 32],
     pub lock_until: i64,
     pub amount: u64,
     pub token_mint: Pubkey,
     pub vault: Pubkey,
    pub dleq_verified: bool,
     pub unlocked: bool,
     pub lock_bump: u8,
 }
 
 impl AtomicLock {
    pub const SIZE: usize = 32 * 11 + 8 + 8 + 1 + 1 + 1;
     pub const LEN: usize = 8 + Self::SIZE;
     pub const SEED_PREFIX: &'static [u8] = b"lock";
     pub const VAULT_SEED_PREFIX: &'static [u8] = b"vault";
 }

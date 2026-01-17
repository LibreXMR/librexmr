use anchor_lang::prelude::*;

#[event]
pub struct Initialized {
    pub lock: Pubkey,
    pub depositor: Pubkey,
    pub token_mint: Pubkey,
    pub amount: u64,
    pub lock_until: i64,
}

#[event]
pub struct Unlocked {
    pub lock: Pubkey,
    pub unlocker: Pubkey,
    pub amount: u64,
}

#[event]
pub struct Refunded {
    pub lock: Pubkey,
    pub depositor: Pubkey,
    pub amount: u64,
}

 use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
 
 pub mod crypto;
 pub mod errors;
pub mod events;
 pub mod instructions;
pub mod logging;
 pub mod state;
 
use crate::state::AtomicLock;
 
declare_id!("GpE4KVAv1Pzc2uhACFU6dAcLac46995Ueh2bhrKngtrM");
 
#[derive(Accounts)]
#[instruction(hashlock: [u8; 32])]
pub struct Initialize<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(
        init,
        payer = depositor,
        space = AtomicLock::LEN,
        seeds = [AtomicLock::SEED_PREFIX, depositor.key().as_ref(), hashlock.as_ref()],
        bump
    )]
    pub atomic_lock: Account<'info, AtomicLock>,
    #[account(
        init,
        payer = depositor,
        seeds = [AtomicLock::VAULT_SEED_PREFIX, atomic_lock.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = atomic_lock
    )]
    pub vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub depositor_token: Account<'info, TokenAccount>,
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct VerifyAndUnlock<'info> {
    #[account(mut)]
    pub unlocker: Signer<'info>,
    #[account(
        mut,
        seeds = [AtomicLock::SEED_PREFIX, atomic_lock.depositor.as_ref(), atomic_lock.hashlock.as_ref()],
        bump = atomic_lock.lock_bump,
        has_one = vault,
    )]
    pub atomic_lock: Account<'info, AtomicLock>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub unlocker_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct VerifyDleq<'info> {
    #[account(
        mut,
        seeds = [AtomicLock::SEED_PREFIX, atomic_lock.depositor.as_ref(), atomic_lock.hashlock.as_ref()],
        bump = atomic_lock.lock_bump,
    )]
    pub atomic_lock: Account<'info, AtomicLock>,
}

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(
        mut,
        seeds = [AtomicLock::SEED_PREFIX, atomic_lock.depositor.as_ref(), atomic_lock.hashlock.as_ref()],
        bump = atomic_lock.lock_bump,
        has_one = vault,
    )]
    pub atomic_lock: Account<'info, AtomicLock>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub depositor_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

 #[program]
 pub mod atomic_lock {
     use super::*;
 
     pub fn initialize(
         ctx: Context<Initialize>,
         hashlock: [u8; 32],
         adaptor_point: [u8; 32],
         second_point: [u8; 32],
         y_point: [u8; 32],
         r1: [u8; 32],
         r2: [u8; 32],
         challenge: [u8; 32],
         response: [u8; 32],
         lock_until: i64,
         amount: u64,
     ) -> Result<()> {
         instructions::initialize::handler(
             ctx,
             hashlock,
             adaptor_point,
             second_point,
             y_point,
             r1,
             r2,
             challenge,
             response,
             lock_until,
             amount,
         )
     }
 
     pub fn verify_and_unlock(ctx: Context<VerifyAndUnlock>, secret: [u8; 32]) -> Result<()> {
         instructions::verify_and_unlock::handler(ctx, secret)
     }
 
    pub fn verify_dleq(ctx: Context<VerifyDleq>) -> Result<()> {
        instructions::verify_dleq::handler(ctx)
    }

     pub fn refund(ctx: Context<Refund>) -> Result<()> {
         instructions::refund::handler(ctx)
     }
 }

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};
 
use crate::crypto::dleq::validate_dleq_transcript_bytes;
 use crate::errors::ErrorCode;
use crate::events::Initialized;
use crate::Initialize;
use crate::logging::debug_log;
 
 pub fn handler(
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
    debug_log("initialize: start");
     let clock = Clock::get()?;
    require!(lock_until > clock.unix_timestamp, ErrorCode::InvalidTimelock);
 
     require!(
         ctx.accounts.depositor_token.mint == ctx.accounts.token_mint.key(),
         ErrorCode::InvalidTokenMint
     );
 
     let dleq_ok = validate_dleq_transcript_bytes(
         &adaptor_point,
         &second_point,
         &y_point,
         &r1,
         &r2,
         &challenge,
         &hashlock,
     )?;
 
     require!(dleq_ok, ErrorCode::InvalidDleqProof);
 
     let lock = &mut ctx.accounts.atomic_lock;
     lock.depositor = ctx.accounts.depositor.key();
     lock.hashlock = hashlock;
     lock.adaptor_point = adaptor_point;
     lock.second_point = second_point;
     lock.y_point = y_point;
     lock.r1 = r1;
     lock.r2 = r2;
     lock.challenge = challenge;
     lock.response = response;
     lock.lock_until = lock_until;
     lock.amount = amount;
     lock.token_mint = ctx.accounts.token_mint.key();
     lock.vault = ctx.accounts.vault.key();
    lock.dleq_verified = false;
     lock.unlocked = false;
     lock.lock_bump = ctx.bumps.atomic_lock;
 
     let cpi_accounts = Transfer {
         from: ctx.accounts.depositor_token.to_account_info(),
         to: ctx.accounts.vault.to_account_info(),
         authority: ctx.accounts.depositor.to_account_info(),
     };
     let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
     token::transfer(cpi_ctx, amount)?;

    emit!(Initialized {
        lock: ctx.accounts.atomic_lock.key(),
        depositor: ctx.accounts.depositor.key(),
        token_mint: ctx.accounts.token_mint.key(),
        amount,
        lock_until,
    });

    debug_log("initialize: complete");
 
     Ok(())
 }

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};
 
 use crate::errors::ErrorCode;
use crate::events::Refunded;
use crate::logging::debug_log;
use crate::state::AtomicLock;
use crate::Refund;
 
 pub fn handler(ctx: Context<Refund>) -> Result<()> {
    debug_log("refund: start");
    let lock = &mut ctx.accounts.atomic_lock;
    let lock_key = lock.key();
    let depositor_key = ctx.accounts.depositor.key();
 
     require!(!lock.unlocked, ErrorCode::AlreadyUnlocked);
     require!(lock.depositor == ctx.accounts.depositor.key(), ErrorCode::UnauthorizedDepositor);
 
     let clock = Clock::get()?;
     require!(
         clock.unix_timestamp >= lock.lock_until,
         ErrorCode::RefundNotAvailable
     );
 
     require!(
         ctx.accounts.depositor_token.mint == lock.token_mint,
         ErrorCode::InvalidTokenMint
     );
 
     let seeds = &[
         AtomicLock::SEED_PREFIX,
         lock.depositor.as_ref(),
         lock.hashlock.as_ref(),
         &[lock.lock_bump],
     ];
     let signer = &[&seeds[..]];
 
     let cpi_accounts = Transfer {
         from: ctx.accounts.vault.to_account_info(),
         to: ctx.accounts.depositor_token.to_account_info(),
         authority: lock.to_account_info(),
     };
     let cpi_ctx =
         CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), cpi_accounts, signer);
     token::transfer(cpi_ctx, lock.amount)?;
 
     lock.unlocked = true;
 
    emit!(Refunded {
        lock: lock_key,
        depositor: depositor_key,
        amount: lock.amount,
    });

    debug_log("refund: complete");

     Ok(())
 }

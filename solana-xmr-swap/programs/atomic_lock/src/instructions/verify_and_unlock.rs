use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};
use crate::errors::ErrorCode;
use crate::events::Unlocked;
use crate::logging::debug_log;
use crate::state::AtomicLock;
use crate::VerifyAndUnlock;
use sha2::Digest;
 
 pub fn handler(ctx: Context<VerifyAndUnlock>, secret: [u8; 32]) -> Result<()> {
    debug_log("verify_and_unlock: start");
    let lock = &mut ctx.accounts.atomic_lock;
    let lock_key = lock.key();
    let unlocker_key = ctx.accounts.unlocker.key();
     require!(!lock.unlocked, ErrorCode::AlreadyUnlocked);
    require!(lock.dleq_verified, ErrorCode::DleqNotVerified);
 
     require!(
         ctx.accounts.unlocker_token.mint == lock.token_mint,
         ErrorCode::InvalidTokenMint
     );
 
    let computed_hash: [u8; 32] = sha2::Sha256::digest(secret).into();
     require!(computed_hash == lock.hashlock, ErrorCode::InvalidSecret);
 
     let seeds = &[
         AtomicLock::SEED_PREFIX,
         lock.depositor.as_ref(),
         lock.hashlock.as_ref(),
         &[lock.lock_bump],
     ];
     let signer = &[&seeds[..]];
 
     let cpi_accounts = Transfer {
         from: ctx.accounts.vault.to_account_info(),
         to: ctx.accounts.unlocker_token.to_account_info(),
         authority: lock.to_account_info(),
     };
     let cpi_ctx =
         CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), cpi_accounts, signer);
     token::transfer(cpi_ctx, lock.amount)?;
 
     lock.unlocked = true;
 
    emit!(Unlocked {
        lock: lock_key,
        unlocker: unlocker_key,
        amount: lock.amount,
    });

    debug_log("verify_and_unlock: complete");

     Ok(())
 }

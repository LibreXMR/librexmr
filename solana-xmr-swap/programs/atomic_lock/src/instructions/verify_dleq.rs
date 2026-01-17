use anchor_lang::prelude::*;

use crate::crypto::dleq::validate_dleq_transcript_bytes;
use crate::errors::ErrorCode;
use crate::logging::debug_log;
use crate::VerifyDleq;

pub fn handler(ctx: Context<VerifyDleq>) -> Result<()> {
    debug_log("verify_dleq: start");
    let lock = &mut ctx.accounts.atomic_lock;

    if lock.dleq_verified {
        debug_log("verify_dleq: already verified");
        return Ok(());
    }

    let dleq_ok = validate_dleq_transcript_bytes(
        &lock.adaptor_point,
        &lock.second_point,
        &lock.y_point,
        &lock.r1,
        &lock.r2,
        &lock.challenge,
        &lock.hashlock,
    )?;

    require!(dleq_ok, ErrorCode::InvalidDleqProof);

    lock.dleq_verified = true;
    debug_log("verify_dleq: complete");
    Ok(())
}

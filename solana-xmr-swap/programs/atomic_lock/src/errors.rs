 use anchor_lang::prelude::*;
 
 #[error_code]
 pub enum ErrorCode {
     #[msg("Invalid DLEQ proof")]
     InvalidDleqProof,
     #[msg("Invalid point encoding")]
     InvalidPointEncoding,
     #[msg("Invalid secret")]
     InvalidSecret,
     #[msg("Already unlocked")]
     AlreadyUnlocked,
     #[msg("Refund not yet available")]
     RefundNotAvailable,
    #[msg("Lock time must be in the future")]
    InvalidTimelock,
     #[msg("Unauthorized depositor")]
     UnauthorizedDepositor,
     #[msg("Invalid token mint")]
     InvalidTokenMint,
    #[msg("DLEQ proof not verified")]
    DleqNotVerified,
 }

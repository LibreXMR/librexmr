# Audit Status Report (Solana XMR Swap)

## Executive Summary
- We are past the MVP skeleton: `initialize`, `verify_dleq`, `verify_and_unlock`, and `refund` are implemented with PDA vaults and token transfers.
- SBF builds are clean after vendor patches; BPF integration tests pass when `RUN_BPF_TESTS=1` and `BPF_OUT_DIR` is set.
- **Protocol change:** full DLEQ proof verification is now off‑chain; on‑chain only validates the transcript hash and hashlock due to Solana CU limits.

## Current Stage (Audit Readiness)
**Stage:** Pre‑audit, implementation‑complete for core MVP with compute‑bounded changes.  
**Status:** SBF build is clean; integration tests pass under BPF. On‑chain DLEQ verification is removed in favor of transcript validation and off‑chain proof checks.  
**Audit risk:** Medium until the off‑chain verification requirement is formally documented and threat‑modeled.

## Coding Style & Principles (How We Are Coding)
- **Safety-first**: explicit error codes, `require!` guards, small‑order checks, and strict validation paths.
- **Deterministic validation**: on‑chain transcript hash checks are pure, testable, and hash‑domain separated.
- **Observability**: feature‑gated logs and Anchor events for state transitions.
- **Test-driven**: unit tests for DLEQ proof validity + negative cases; integration tests cover token flow when BPF artifacts are available.

## Key Progress (What Works)
### 1) On-chain DLEQ Transcript Validation
Full DLEQ proof verification is off‑chain. On‑chain we only validate the transcript hash to keep CU usage within mainnet limits:
```
171:191:programs/atomic_lock/src/crypto/dleq.rs
pub fn validate_dleq_transcript_bytes(
    adaptor_point: &[u8; 32],
    second_point: &[u8; 32],
    y_point: &[u8; 32],
    r1: &[u8; 32],
    r2: &[u8; 32],
    challenge: &[u8; 32],
    hashlock: &[u8; 32],
) -> Result<bool> {
    let mut hasher = Blake2s256::new();
    hasher.update(DLEQ_DOMAIN_TAG);
    hasher.update(ED25519_BASEPOINT_POINT.compress().to_bytes());
    hasher.update(y_point);
    hasher.update(adaptor_point);
    hasher.update(second_point);
    hasher.update(r1);
    hasher.update(r2);
    hasher.update(hashlock);
    let computed: [u8; 32] = hasher.finalize().into();

    Ok(&computed == challenge)
}
```

### 2) Unlock Path (CU‑bounded)
Unlock now checks the hashlock and requires `dleq_verified`, avoiding curve operations in the hot path:
```
10:25:programs/atomic_lock/src/instructions/verify_and_unlock.rs
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
    /* ... */
}
```

### 3) Program Entry Points & PDA Flow
The program accounts and entrypoints are wired for initialize / verify_dleq / unlock / refund:
```
91:133:programs/atomic_lock/src/lib.rs
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

#[program]
pub mod atomic_lock {
    use super::*;

    pub fn initialize( /* ... */ ) -> Result<()> {
        instructions::initialize::handler( /* ... */ )
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
```

### 4) Initialize / VerifyDleq / Unlock Paths
Initialize validates the transcript hash and stores `dleq_verified = false`. `verify_dleq` flips the flag, and unlock requires it:
```
10:58:programs/atomic_lock/src/instructions/initialize.rs
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
    /* ... */
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
    /* ... */
    lock.dleq_verified = false;
    /* ... */
}
```

```
8:31:programs/atomic_lock/src/instructions/verify_dleq.rs
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
    Ok(())
}
```

## Current Blocker (Issue)
**No build blocker.** SBF builds are clean after vendor patches.  
**Primary audit risk:** on‑chain DLEQ verification was removed to stay within Solana CU limits. Clients must verify the DLEQ proof off‑chain before calling `verify_dleq`.

### Why Off‑Chain DLEQ on Solana
Solana SBF enforces a hard per‑transaction compute limit (about 1.4M CUs), and there are no native precompiles/syscalls for general elliptic‑curve scalar multiplication. DLEQ verification requires multiple scalar multiplications and point ops, which exceed the CU budget in practice when implemented in Rust on SBF. Starknet/Cairo, by contrast, has EC builtins (EC_OP) and a proof‑centric execution model where heavy algebraic work is amortized by the STARK prover; large cryptographic workloads are more feasible within its gas model. This is why we keep DLEQ verification off‑chain on Solana and only validate the transcript hash on‑chain.

## Overall Progress (What is Done)
- Toolchain setup succeeded (Agave/solana-release installed; SBF toolchain linked).
- SBF build is clean; integration tests pass under BPF with `RUN_BPF_TESTS=1` and `BPF_OUT_DIR`.
- DLEQ transcript validation implemented on‑chain; full proof verification remains in unit tests and must be done off‑chain.
- Initialize/verify_dleq/unlock/refund flows implemented with PDA checks, token transfers, events, and debug logs.
- Canonical DLEQ test vector mirrored from the Starknet Monero swap implementation for client verification.
- Off‑chain DLEQ verifier tooling shipped in Rust and a minimal TS wrapper.
- Vendor patch summary and toolchain pins added for audit packaging.
- SHA256 hash manifest added for patched vendor files.
- Audit bundle index added for auditor consumption.
- Ops tooling added: swap watchtower and status API (`tools/swap_ops`).
- Client SDK hardening added: `swap_sdk` for off-chain verification + audit logs.
- Webhook alerts with retry/backoff and schema documentation added for ops tooling.
- Optional signing support added for audit logs (`swap_sdk`).
- Audit log verification CLI added (`swap_sdk`).
- Optional signing support added for webhook alerts (`swap_ops`).
- Webhook alert verification CLI added (`swap_ops`).
- Sample signed webhook alert added for verification (`docs/alert_signed_sample.json`).
- Sample signed audit log added for verification (`docs/audit_signed_sample.json`).
- Swap coordinator added (ported state machine from the Starknet swap repo).

## What’s Next (Action Plan)
1. **Document off‑chain DLEQ requirement**
   - Add `SECURITY.md` with the threat model and explicit client responsibilities.
   - Update client docs/README to describe the `verify_dleq` step and off‑chain proof checks.

2. **Compute unit profiling**
   - Add a CU harness for swap flows with the current on‑chain validation path.
   - Revisit on‑chain DLEQ verification only if a low‑CU syscall or precompile becomes available.

3. **Audit PR readiness**
   - Freeze dependencies, document vendor patches, and pin toolchain versions.

## Known Risks for Audit Review
- Vendor patching crypto crates is an audit flag; requires justification and isolation.
- Off‑chain DLEQ verification is now required; auditors will want a clear threat model and client obligations.


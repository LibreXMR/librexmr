use atomic_lock::crypto::dleq::compute_challenge_bytes;
use atomic_lock::errors::ErrorCode;
use atomic_lock::state::AtomicLock;
use anchor_lang::prelude::AccountDeserialize;
use anchor_lang::{InstructionData, ToAccountMetas};
use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
use curve25519_dalek::scalar::Scalar;
use sha2::Digest;
use solana_program_test::{BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::InstructionError;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::system_instruction;
use solana_sdk::sysvar;
use solana_sdk::transaction::{Transaction, TransactionError};
use spl_token::state::Account as TokenAccountState;

const TOKEN_DECIMALS: u8 = 9;
const SWAP_AMOUNT: u64 = 1_000_000_000;
const INIT_FEE_BUFFER_LAMPORTS: u64 = 1_000_000;
const COMPUTE_UNIT_LIMIT: u32 = 1_400_000;

struct DleqProofData {
    secret: [u8; 32],
    hashlock: [u8; 32],
    adaptor_point: [u8; 32],
    second_point: [u8; 32],
    y_point: [u8; 32],
    r1: [u8; 32],
    r2: [u8; 32],
    challenge: [u8; 32],
    response: [u8; 32],
}

fn build_dleq_proof() -> DleqProofData {
    let t = Scalar::random(&mut rand::rngs::OsRng);
    let k = Scalar::random(&mut rand::rngs::OsRng);
    let y = Scalar::random(&mut rand::rngs::OsRng);

    let y_point = &ED25519_BASEPOINT_POINT * &y;
    let t_point = &ED25519_BASEPOINT_POINT * &t;
    let u_point = &y_point * &t;

    let r1_point = &ED25519_BASEPOINT_POINT * &k;
    let r2_point = &y_point * &k;

    let hashlock: [u8; 32] = sha2::Sha256::digest(t.to_bytes()).into();

    let challenge = compute_challenge_bytes(
        &ED25519_BASEPOINT_POINT,
        &y_point,
        &t_point,
        &u_point,
        &r1_point,
        &r2_point,
        &hashlock,
    );

    let c = Scalar::from_bytes_mod_order(challenge);
    let s = k + c * t;

    DleqProofData {
        secret: t.to_bytes(),
        hashlock,
        adaptor_point: t_point.compress().to_bytes(),
        second_point: u_point.compress().to_bytes(),
        y_point: y_point.compress().to_bytes(),
        r1: r1_point.compress().to_bytes(),
        r2: r2_point.compress().to_bytes(),
        challenge,
        response: s.to_bytes(),
    }
}

async fn create_mint(ctx: &mut ProgramTestContext, mint_authority: &Keypair) -> Pubkey {
    let mint = Keypair::new();
    let rent = ctx.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(spl_token::state::Mint::LEN);
    let create_ix = system_instruction::create_account(
        &ctx.payer.pubkey(),
        &mint.pubkey(),
        lamports,
        spl_token::state::Mint::LEN as u64,
        &spl_token::ID,
    );
    let init_ix = spl_token::instruction::initialize_mint(
        &spl_token::ID,
        &mint.pubkey(),
        &mint_authority.pubkey(),
        None,
        TOKEN_DECIMALS,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[create_ix, init_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &mint],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();
    mint.pubkey()
}

async fn create_token_account(
    ctx: &mut ProgramTestContext,
    owner: &Keypair,
    mint: Pubkey,
) -> Pubkey {
    let account = Keypair::new();
    let rent = ctx.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(TokenAccountState::LEN);
    let create_ix = system_instruction::create_account(
        &ctx.payer.pubkey(),
        &account.pubkey(),
        lamports,
        TokenAccountState::LEN as u64,
        &spl_token::ID,
    );
    let init_ix = spl_token::instruction::initialize_account(
        &spl_token::ID,
        &account.pubkey(),
        &mint,
        &owner.pubkey(),
    )
    .unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[create_ix, init_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &account],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();
    account.pubkey()
}

async fn depositor_top_up_ix(
    ctx: &mut ProgramTestContext,
    depositor: Pubkey,
) -> Option<solana_sdk::instruction::Instruction> {
    let rent = ctx.banks_client.get_rent().await.unwrap();
    let required = rent.minimum_balance(AtomicLock::LEN)
        + rent.minimum_balance(TokenAccountState::LEN)
        + INIT_FEE_BUFFER_LAMPORTS;
    let current = ctx.banks_client.get_balance(depositor).await.unwrap();
    if current >= required {
        return None;
    }
    Some(system_instruction::transfer(
        &ctx.payer.pubkey(),
        &depositor,
        required - current,
    ))
}

fn compute_budget_ix() -> solana_sdk::instruction::Instruction {
    ComputeBudgetInstruction::set_compute_unit_limit(COMPUTE_UNIT_LIMIT)
}

async fn mint_tokens(
    ctx: &mut ProgramTestContext,
    mint: Pubkey,
    mint_authority: &Keypair,
    destination: Pubkey,
    amount: u64,
) {
    let ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        &mint,
        &destination,
        &mint_authority.pubkey(),
        &[],
        amount,
    )
    .unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, mint_authority],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();
}

async fn fetch_token_amount(ctx: &mut ProgramTestContext, account: Pubkey) -> u64 {
    let account = ctx.banks_client.get_account(account).await.unwrap().unwrap();
    let token_account = TokenAccountState::unpack(&account.data).unwrap();
    token_account.amount
}

async fn fetch_lock(ctx: &mut ProgramTestContext, lock: Pubkey) -> AtomicLock {
    let account = ctx.banks_client.get_account(lock).await.unwrap().unwrap();
    let mut data = account.data.as_slice();
    AtomicLock::try_deserialize(&mut data).unwrap()
}

fn assert_anchor_error(err: BanksClientError, expected: ErrorCode) {
    let expected: u32 = expected.into();
    match err {
        BanksClientError::TransactionError(TransactionError::InstructionError(
            _,
            InstructionError::Custom(code),
        )) => assert_eq!(code, expected),
        other => panic!("unexpected error: {other:?}"),
    }
}

fn should_run_bpf_tests() -> bool {
    if std::env::var("BPF_OUT_DIR").is_err() {
        return false;
    }
    let run_bpf = matches!(
        std::env::var("RUN_BPF_TESTS").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    );
    run_bpf || std::env::var("CI").is_ok()
}

async fn program_context() -> ProgramTestContext {
    let mut program = ProgramTest::new("atomic_lock", atomic_lock::ID, None);
    program.set_compute_max_units(COMPUTE_UNIT_LIMIT as u64);
    program.prefer_bpf(true);
    program.start_with_context().await
}

async fn initialize_swap(
    ctx: &mut ProgramTestContext,
    proof: &DleqProofData,
    depositor: &Keypair,
    depositor_token: Pubkey,
    token_mint: Pubkey,
    lock_until: i64,
    amount: u64,
) -> (Pubkey, Pubkey) {
    let top_up_ix = depositor_top_up_ix(ctx, depositor.pubkey()).await;

    let (lock_pda, _bump) = Pubkey::find_program_address(
        &[
            AtomicLock::SEED_PREFIX,
            depositor.pubkey().as_ref(),
            proof.hashlock.as_ref(),
        ],
        &atomic_lock::ID,
    );
    let (vault_pda, _vault_bump) =
        Pubkey::find_program_address(&[AtomicLock::VAULT_SEED_PREFIX, lock_pda.as_ref()], &atomic_lock::ID);

    let ix = solana_sdk::instruction::Instruction {
        program_id: atomic_lock::ID,
        accounts: atomic_lock::accounts::Initialize {
            depositor: depositor.pubkey(),
            atomic_lock: lock_pda,
            vault: vault_pda,
            depositor_token,
            token_mint,
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
            rent: sysvar::rent::ID,
        }
        .to_account_metas(None),
        data: atomic_lock::instruction::Initialize {
            hashlock: proof.hashlock,
            adaptor_point: proof.adaptor_point,
            second_point: proof.second_point,
            y_point: proof.y_point,
            r1: proof.r1,
            r2: proof.r2,
            challenge: proof.challenge,
            response: proof.response,
            lock_until,
            amount,
        }
        .data(),
    };

    let compute_ix = compute_budget_ix();
    let mut instructions = vec![compute_ix];
    if let Some(top_up_ix) = top_up_ix {
        instructions.push(top_up_ix);
    }
    instructions.push(ix);
    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, depositor],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();
    (lock_pda, vault_pda)
}

async fn verify_dleq(ctx: &mut ProgramTestContext, lock_pda: Pubkey) {
    let ix = solana_sdk::instruction::Instruction {
        program_id: atomic_lock::ID,
        accounts: atomic_lock::accounts::VerifyDleq {
            atomic_lock: lock_pda,
        }
        .to_account_metas(None),
        data: atomic_lock::instruction::VerifyDleq {}.data(),
    };

    let compute_ix = compute_budget_ix();
    let tx = Transaction::new_signed_with_payer(
        &[compute_ix, ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();
}

async fn warp_to_timestamp(ctx: &mut ProgramTestContext, target: i64) {
    loop {
        let clock: solana_sdk::clock::Clock = ctx.banks_client.get_sysvar().await.unwrap();
        if clock.unix_timestamp >= target {
            break;
        }
        ctx.warp_to_slot(clock.slot + 10).unwrap();
    }
}

#[tokio::test]
async fn initialize_then_unlock_happy_path() {
    if !should_run_bpf_tests() {
        eprintln!(
            "Skipping: set RUN_BPF_TESTS=1 (CI runs by default) and BPF_OUT_DIR to enable."
        );
        return;
    }
    let mut ctx = program_context().await;
    let depositor = Keypair::new();
    let unlocker = Keypair::new();
    let proof = build_dleq_proof();

    let mint = create_mint(&mut ctx, &depositor).await;
    let depositor_token = create_token_account(&mut ctx, &depositor, mint).await;
    let unlocker_token = create_token_account(&mut ctx, &unlocker, mint).await;
    mint_tokens(&mut ctx, mint, &depositor, depositor_token, SWAP_AMOUNT).await;

    let clock: solana_sdk::clock::Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let lock_until = clock.unix_timestamp + 100;

    let (lock_pda, _vault_pda) = initialize_swap(
        &mut ctx,
        &proof,
        &depositor,
        depositor_token,
        mint,
        lock_until,
        SWAP_AMOUNT,
    )
    .await;

    verify_dleq(&mut ctx, lock_pda).await;

    let ix = solana_sdk::instruction::Instruction {
        program_id: atomic_lock::ID,
        accounts: atomic_lock::accounts::VerifyAndUnlock {
            unlocker: unlocker.pubkey(),
            atomic_lock: lock_pda,
            vault: Pubkey::find_program_address(
                &[AtomicLock::VAULT_SEED_PREFIX, lock_pda.as_ref()],
                &atomic_lock::ID,
            )
            .0,
            unlocker_token,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
        data: atomic_lock::instruction::VerifyAndUnlock { secret: proof.secret }.data(),
    };

    let compute_ix = compute_budget_ix();
    let tx = Transaction::new_signed_with_payer(
        &[compute_ix, ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &unlocker],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    assert_eq!(fetch_token_amount(&mut ctx, unlocker_token).await, SWAP_AMOUNT);
    assert_eq!(fetch_lock(&mut ctx, lock_pda).await.unlocked, true);
}

#[tokio::test]
async fn unlock_with_wrong_secret_fails() {
    if !should_run_bpf_tests() {
        eprintln!(
            "Skipping: set RUN_BPF_TESTS=1 (CI runs by default) and BPF_OUT_DIR to enable."
        );
        return;
    }
    let mut ctx = program_context().await;
    let depositor = Keypair::new();
    let unlocker = Keypair::new();
    let proof = build_dleq_proof();

    let mint = create_mint(&mut ctx, &depositor).await;
    let depositor_token = create_token_account(&mut ctx, &depositor, mint).await;
    let unlocker_token = create_token_account(&mut ctx, &unlocker, mint).await;
    mint_tokens(&mut ctx, mint, &depositor, depositor_token, SWAP_AMOUNT).await;

    let clock: solana_sdk::clock::Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let lock_until = clock.unix_timestamp + 100;

    let (lock_pda, _vault_pda) = initialize_swap(
        &mut ctx,
        &proof,
        &depositor,
        depositor_token,
        mint,
        lock_until,
        SWAP_AMOUNT,
    )
    .await;

    verify_dleq(&mut ctx, lock_pda).await;

    let mut bad_secret = proof.secret;
    bad_secret[0] ^= 0x01;

    let ix = solana_sdk::instruction::Instruction {
        program_id: atomic_lock::ID,
        accounts: atomic_lock::accounts::VerifyAndUnlock {
            unlocker: unlocker.pubkey(),
            atomic_lock: lock_pda,
            vault: Pubkey::find_program_address(
                &[AtomicLock::VAULT_SEED_PREFIX, lock_pda.as_ref()],
                &atomic_lock::ID,
            )
            .0,
            unlocker_token,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
        data: atomic_lock::instruction::VerifyAndUnlock { secret: bad_secret }.data(),
    };

    let compute_ix = compute_budget_ix();
    let tx = Transaction::new_signed_with_payer(
        &[compute_ix, ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &unlocker],
        ctx.last_blockhash,
    );
    let err = ctx.banks_client.process_transaction(tx).await.unwrap_err();
    assert_anchor_error(err, ErrorCode::InvalidSecret);
}

#[tokio::test]
async fn invalid_dleq_proof_is_rejected() {
    if !should_run_bpf_tests() {
        eprintln!(
            "Skipping: set RUN_BPF_TESTS=1 (CI runs by default) and BPF_OUT_DIR to enable."
        );
        return;
    }
    let mut ctx = program_context().await;
    let depositor = Keypair::new();
    let proof = build_dleq_proof();

    let mint = create_mint(&mut ctx, &depositor).await;
    let depositor_token = create_token_account(&mut ctx, &depositor, mint).await;
    mint_tokens(&mut ctx, mint, &depositor, depositor_token, SWAP_AMOUNT).await;

    let clock: solana_sdk::clock::Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let lock_until = clock.unix_timestamp + 100;

    let (lock_pda, _vault_pda) = Pubkey::find_program_address(
        &[
            AtomicLock::SEED_PREFIX,
            depositor.pubkey().as_ref(),
            proof.hashlock.as_ref(),
        ],
        &atomic_lock::ID,
    );
    let (vault_pda, _vault_bump) =
        Pubkey::find_program_address(&[AtomicLock::VAULT_SEED_PREFIX, lock_pda.as_ref()], &atomic_lock::ID);

    let mut bad_challenge = proof.challenge;
    bad_challenge[1] ^= 0x22;

    let top_up_ix = depositor_top_up_ix(&mut ctx, depositor.pubkey()).await;

    let ix = solana_sdk::instruction::Instruction {
        program_id: atomic_lock::ID,
        accounts: atomic_lock::accounts::Initialize {
            depositor: depositor.pubkey(),
            atomic_lock: lock_pda,
            vault: vault_pda,
            depositor_token,
            token_mint: mint,
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
            rent: sysvar::rent::ID,
        }
        .to_account_metas(None),
        data: atomic_lock::instruction::Initialize {
            hashlock: proof.hashlock,
            adaptor_point: proof.adaptor_point,
            second_point: proof.second_point,
            y_point: proof.y_point,
            r1: proof.r1,
            r2: proof.r2,
            challenge: bad_challenge,
            response: proof.response,
            lock_until,
            amount: SWAP_AMOUNT,
        }
        .data(),
    };

    let compute_ix = compute_budget_ix();
    let mut instructions = vec![compute_ix];
    if let Some(top_up_ix) = top_up_ix {
        instructions.push(top_up_ix);
    }
    instructions.push(ix);
    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &depositor],
        ctx.last_blockhash,
    );
    let err = ctx.banks_client.process_transaction(tx).await.unwrap_err();
    assert_anchor_error(err, ErrorCode::InvalidDleqProof);
}

#[tokio::test]
async fn refund_before_timelock_fails() {
    if !should_run_bpf_tests() {
        eprintln!(
            "Skipping: set RUN_BPF_TESTS=1 (CI runs by default) and BPF_OUT_DIR to enable."
        );
        return;
    }
    let mut ctx = program_context().await;
    let depositor = Keypair::new();
    let proof = build_dleq_proof();

    let mint = create_mint(&mut ctx, &depositor).await;
    let depositor_token = create_token_account(&mut ctx, &depositor, mint).await;
    mint_tokens(&mut ctx, mint, &depositor, depositor_token, SWAP_AMOUNT).await;

    let clock: solana_sdk::clock::Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let lock_until = clock.unix_timestamp + 1_000;

    let (lock_pda, _vault_pda) = initialize_swap(
        &mut ctx,
        &proof,
        &depositor,
        depositor_token,
        mint,
        lock_until,
        SWAP_AMOUNT,
    )
    .await;

    let ix = solana_sdk::instruction::Instruction {
        program_id: atomic_lock::ID,
        accounts: atomic_lock::accounts::Refund {
            depositor: depositor.pubkey(),
            atomic_lock: lock_pda,
            vault: Pubkey::find_program_address(
                &[AtomicLock::VAULT_SEED_PREFIX, lock_pda.as_ref()],
                &atomic_lock::ID,
            )
            .0,
            depositor_token,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
        data: atomic_lock::instruction::Refund {}.data(),
    };

    let compute_ix = compute_budget_ix();
    let tx = Transaction::new_signed_with_payer(
        &[compute_ix, ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &depositor],
        ctx.last_blockhash,
    );
    let err = ctx.banks_client.process_transaction(tx).await.unwrap_err();
    assert_anchor_error(err, ErrorCode::RefundNotAvailable);
}

#[tokio::test]
async fn unlock_after_refund_fails() {
    if !should_run_bpf_tests() {
        eprintln!(
            "Skipping: set RUN_BPF_TESTS=1 (CI runs by default) and BPF_OUT_DIR to enable."
        );
        return;
    }
    let mut ctx = program_context().await;
    let depositor = Keypair::new();
    let unlocker = Keypair::new();
    let proof = build_dleq_proof();

    let mint = create_mint(&mut ctx, &depositor).await;
    let depositor_token = create_token_account(&mut ctx, &depositor, mint).await;
    let unlocker_token = create_token_account(&mut ctx, &unlocker, mint).await;
    mint_tokens(&mut ctx, mint, &depositor, depositor_token, SWAP_AMOUNT).await;

    let clock: solana_sdk::clock::Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let lock_until = clock.unix_timestamp + 1;

    let (lock_pda, _vault_pda) = initialize_swap(
        &mut ctx,
        &proof,
        &depositor,
        depositor_token,
        mint,
        lock_until,
        SWAP_AMOUNT,
    )
    .await;

    warp_to_timestamp(&mut ctx, lock_until).await;

    let refund_ix = solana_sdk::instruction::Instruction {
        program_id: atomic_lock::ID,
        accounts: atomic_lock::accounts::Refund {
            depositor: depositor.pubkey(),
            atomic_lock: lock_pda,
            vault: Pubkey::find_program_address(
                &[AtomicLock::VAULT_SEED_PREFIX, lock_pda.as_ref()],
                &atomic_lock::ID,
            )
            .0,
            depositor_token,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
        data: atomic_lock::instruction::Refund {}.data(),
    };
    let compute_ix = compute_budget_ix();
    let refund_tx = Transaction::new_signed_with_payer(
        &[compute_ix, refund_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &depositor],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(refund_tx).await.unwrap();

    let unlock_ix = solana_sdk::instruction::Instruction {
        program_id: atomic_lock::ID,
        accounts: atomic_lock::accounts::VerifyAndUnlock {
            unlocker: unlocker.pubkey(),
            atomic_lock: lock_pda,
            vault: Pubkey::find_program_address(
                &[AtomicLock::VAULT_SEED_PREFIX, lock_pda.as_ref()],
                &atomic_lock::ID,
            )
            .0,
            unlocker_token,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
        data: atomic_lock::instruction::VerifyAndUnlock { secret: proof.secret }.data(),
    };
    let compute_ix = compute_budget_ix();
    let unlock_tx = Transaction::new_signed_with_payer(
        &[compute_ix, unlock_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &unlocker],
        ctx.last_blockhash,
    );
    let err = ctx.banks_client.process_transaction(unlock_tx).await.unwrap_err();
    assert_anchor_error(err, ErrorCode::AlreadyUnlocked);
}

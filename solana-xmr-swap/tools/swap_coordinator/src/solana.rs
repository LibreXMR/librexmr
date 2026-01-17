//! Solana RPC client implementation for swap coordinator.

use anyhow::{Context, Result};
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;

use anchor_lang::{InstructionData, ToAccountMetas};
use std::str::FromStr;

use crate::driver::SolanaClient;

pub struct SolanaSwapClient {
    rpc: RpcClient,
    program_id: Pubkey,
    depositor: Keypair,
    unlocker: Keypair,
    token_mint: Pubkey,
    amount: u64,
    hashlock: [u8; 32],
    adaptor_point: [u8; 32],
    second_point: [u8; 32],
    y_point: [u8; 32],
    r1: [u8; 32],
    r2: [u8; 32],
    challenge: [u8; 32],
    response: [u8; 32],
}

impl SolanaSwapClient {
    pub fn new(
        rpc_url: &str,
        program_id: Pubkey,
        depositor: Keypair,
        unlocker: Keypair,
        token_mint: Pubkey,
        amount: u64,
        hashlock: [u8; 32],
        adaptor_point: [u8; 32],
        second_point: [u8; 32],
        y_point: [u8; 32],
        r1: [u8; 32],
        r2: [u8; 32],
        challenge: [u8; 32],
        response: [u8; 32],
    ) -> Self {
        Self {
            rpc: RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed()),
            program_id,
            depositor,
            unlocker,
            token_mint,
            amount,
            hashlock,
            adaptor_point,
            second_point,
            y_point,
            r1,
            r2,
            challenge,
            response,
        }
    }

    pub fn depositor_pubkey(&self) -> Pubkey {
        self.depositor.pubkey()
    }

    pub fn unlocker_pubkey(&self) -> Pubkey {
        self.unlocker.pubkey()
    }

    fn ensure_associated_token_account(
        &self,
        owner: &Pubkey,
    ) -> Result<(Pubkey, Option<Instruction>)> {
        let ata = get_associated_token_address(owner, &self.token_mint);
        if self.rpc.get_account(&ata).is_ok() {
            return Ok((ata, None));
        }
        let ix = create_associated_token_account(
            &self.depositor.pubkey(),
            owner,
            &self.token_mint,
            &spl_token::ID,
        );
        Ok((ata, Some(ix)))
    }

    fn sign_and_send(
        &self,
        ixs: Vec<Instruction>,
        signers: &[&Keypair],
        payer: &Pubkey,
    ) -> Result<Signature> {
        let blockhash = self.rpc.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(&ixs, Some(payer), signers, blockhash);
        let sig = self
            .rpc
            .send_and_confirm_transaction(&tx)
            .context("send transaction failed")?;
        Ok(sig)
    }

    fn derive_lock_pda(&self) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"lock", self.depositor.pubkey().as_ref(), &self.hashlock],
            &self.program_id,
        )
    }

    fn derive_vault_pda(&self, lock_pda: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"vault", lock_pda.as_ref()], &self.program_id)
    }
}

#[async_trait]
impl SolanaClient for SolanaSwapClient {
    async fn initialize(&self, lock_duration_secs: u64) -> Result<(String, String, i64, String)> {
        let (lock_pda, _) = self.derive_lock_pda();
        let (vault_pda, _) = self.derive_vault_pda(&lock_pda);

        let now = self.get_block_timestamp().await?;
        let lock_until = now + lock_duration_secs as i64;

        let (depositor_token, depositor_token_ix) =
            self.ensure_associated_token_account(&self.depositor.pubkey())?;

        let ix = atomic_lock::instruction::Initialize {
            hashlock: self.hashlock,
            adaptor_point: self.adaptor_point,
            second_point: self.second_point,
            y_point: self.y_point,
            r1: self.r1,
            r2: self.r2,
            challenge: self.challenge,
            response: self.response,
            lock_until,
            amount: self.amount,
        }
        .data();

        let accounts = atomic_lock::accounts::Initialize {
            depositor: self.depositor.pubkey(),
            atomic_lock: lock_pda,
            vault: vault_pda,
            depositor_token,
            token_mint: self.token_mint,
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
            rent: solana_sdk::sysvar::rent::ID,
        }
        .to_account_metas(None);

        let mut instructions = vec![];
        if let Some(ix) = depositor_token_ix {
            instructions.push(ix);
        }
        instructions.push(Instruction {
            program_id: self.program_id,
            accounts,
            data: ix,
        });

        let sig = self.sign_and_send(instructions, &[&self.depositor], &self.depositor.pubkey())?;
        Ok((
            lock_pda.to_string(),
            vault_pda.to_string(),
            lock_until,
            sig.to_string(),
        ))
    }

    async fn verify_dleq(&self, lock_pda: &str) -> Result<String> {
        let lock_pda = Pubkey::from_str(lock_pda).context("invalid lock PDA")?;
        let ix = atomic_lock::instruction::VerifyDleq {}.data();
        let accounts = atomic_lock::accounts::VerifyDleq { atomic_lock: lock_pda }
            .to_account_metas(None);
        let sig = self.sign_and_send(
            vec![Instruction {
                program_id: self.program_id,
                accounts,
                data: ix,
            }],
            &[&self.depositor],
            &self.depositor.pubkey(),
        )?;
        Ok(sig.to_string())
    }

    async fn unlock(&self, lock_pda: &str, vault: &str, secret: [u8; 32]) -> Result<String> {
        let lock_pda = Pubkey::from_str(lock_pda).context("invalid lock PDA")?;
        let vault = Pubkey::from_str(vault).context("invalid vault PDA")?;
        let (unlocker_token, unlocker_token_ix) =
            self.ensure_associated_token_account(&self.unlocker.pubkey())?;

        let ix = atomic_lock::instruction::VerifyAndUnlock { secret }.data();
        let accounts = atomic_lock::accounts::VerifyAndUnlock {
            unlocker: self.unlocker.pubkey(),
            atomic_lock: lock_pda,
            vault,
            unlocker_token,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);

        let mut instructions = vec![];
        if let Some(ix) = unlocker_token_ix {
            instructions.push(ix);
        }
        instructions.push(Instruction {
            program_id: self.program_id,
            accounts,
            data: ix,
        });

        let sig = self.sign_and_send(instructions, &[&self.unlocker], &self.unlocker.pubkey())?;
        Ok(sig.to_string())
    }

    async fn refund(&self, lock_pda: &str, vault: &str) -> Result<String> {
        let lock_pda = Pubkey::from_str(lock_pda).context("invalid lock PDA")?;
        let vault = Pubkey::from_str(vault).context("invalid vault PDA")?;
        let (depositor_token, depositor_token_ix) =
            self.ensure_associated_token_account(&self.depositor.pubkey())?;

        let ix = atomic_lock::instruction::Refund {}.data();
        let accounts = atomic_lock::accounts::Refund {
            depositor: self.depositor.pubkey(),
            atomic_lock: lock_pda,
            vault,
            depositor_token,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);

        let mut instructions = vec![];
        if let Some(ix) = depositor_token_ix {
            instructions.push(ix);
        }
        instructions.push(Instruction {
            program_id: self.program_id,
            accounts,
            data: ix,
        });

        let sig = self.sign_and_send(instructions, &[&self.depositor], &self.depositor.pubkey())?;
        Ok(sig.to_string())
    }

    async fn get_block_timestamp(&self) -> Result<i64> {
        let slot = self.rpc.get_slot()?;
        match self.rpc.get_block_time(slot) {
            Ok(block_time) => Ok(block_time),
            Err(_) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                Ok(now)
            }
        }
    }
}


//! Transaction builder.

use super::PriorityLevel;
use anyhow::{Context, Result};
use solana_sdk::hash::Hash;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::Message;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;

/// Builder for constructing transactions.
pub struct TransactionBuilder {
    /// Instructions to include.
    instructions: Vec<Instruction>,
    /// Compute unit limit.
    compute_units: Option<u32>,
    /// Priority level.
    priority: PriorityLevel,
    /// Recent blockhash.
    blockhash: Option<Hash>,
    /// Fee payer.
    fee_payer: Option<solana_sdk::pubkey::Pubkey>,
}

impl TransactionBuilder {
    /// Creates a new transaction builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            compute_units: None,
            priority: PriorityLevel::default(),
            blockhash: None,
            fee_payer: None,
        }
    }

    /// Adds an instruction.
    #[must_use]
    pub fn add_instruction(mut self, instruction: Instruction) -> Self {
        self.instructions.push(instruction);
        self
    }

    /// Adds multiple instructions.
    #[must_use]
    pub fn add_instructions(mut self, instructions: Vec<Instruction>) -> Self {
        self.instructions.extend(instructions);
        self
    }

    /// Sets the compute unit limit.
    #[must_use]
    pub fn with_compute_units(mut self, units: u32) -> Self {
        self.compute_units = Some(units);
        self
    }

    /// Sets the priority level.
    #[must_use]
    pub fn with_priority(mut self, priority: PriorityLevel) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the recent blockhash.
    #[must_use]
    pub fn with_blockhash(mut self, blockhash: Hash) -> Self {
        self.blockhash = Some(blockhash);
        self
    }

    /// Sets the fee payer.
    #[must_use]
    pub fn with_fee_payer(mut self, payer: solana_sdk::pubkey::Pubkey) -> Self {
        self.fee_payer = Some(payer);
        self
    }

    /// Builds the transaction.
    pub fn build(self, signers: &[&Keypair]) -> Result<Transaction> {
        let blockhash = self.blockhash.context("Blockhash not set")?;

        let fee_payer = self
            .fee_payer
            .or_else(|| signers.first().map(|s| s.pubkey()))
            .context("Fee payer not set")?;

        // Build instructions with compute budget
        let mut all_instructions = Vec::new();

        // Note: Compute budget instructions would be added here
        // In solana-sdk 3.x, these are in a separate crate
        // For now, we skip compute budget instructions
        let _ = self.compute_units;
        let _ = self.priority;

        // Add user instructions
        all_instructions.extend(self.instructions);

        // Create message
        let message = Message::new(&all_instructions, Some(&fee_payer));

        // Create and sign transaction
        let mut transaction = Transaction::new_unsigned(message);
        transaction.partial_sign(signers, blockhash);

        Ok(transaction)
    }

    /// Returns the estimated compute units.
    #[must_use]
    pub fn estimated_compute_units(&self) -> u32 {
        self.compute_units.unwrap_or(200_000)
    }
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let builder = TransactionBuilder::new()
            .with_compute_units(100_000)
            .with_priority(PriorityLevel::High);

        assert_eq!(builder.estimated_compute_units(), 100_000);
    }

    #[test]
    fn test_add_instruction() {
        // Create a simple instruction
        let instruction =
            Instruction::new_with_bytes(solana_sdk::pubkey::Pubkey::new_unique(), &[], vec![]);

        let builder = TransactionBuilder::new().add_instruction(instruction);

        assert_eq!(builder.instructions.len(), 1);
    }
}

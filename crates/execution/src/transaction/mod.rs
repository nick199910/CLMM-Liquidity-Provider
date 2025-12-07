//! Transaction building and management.
//!
//! Provides transaction lifecycle handling including:
//! - Transaction building
//! - Priority fee estimation
//! - Simulation
//! - Confirmation tracking

mod builder;
mod manager;

pub use builder::*;
pub use manager::*;

use solana_sdk::signature::Signature;
use std::time::Duration;

/// Transaction status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Transaction is being built.
    Building,
    /// Transaction is being simulated.
    Simulating,
    /// Transaction is ready to send.
    Ready,
    /// Transaction has been sent.
    Sent(Signature),
    /// Transaction is confirmed.
    Confirmed(Signature),
    /// Transaction failed.
    Failed(String),
}

/// Transaction result.
#[derive(Debug, Clone)]
pub struct TransactionResult {
    /// Transaction signature.
    pub signature: Signature,
    /// Slot when confirmed.
    pub slot: u64,
    /// Time to confirmation.
    pub confirmation_time: Duration,
    /// Compute units used.
    pub compute_units: Option<u64>,
    /// Fee paid in lamports.
    pub fee: u64,
}

/// Priority fee level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PriorityLevel {
    /// Low priority (slower, cheaper).
    Low,
    /// Medium priority (balanced).
    #[default]
    Medium,
    /// High priority (faster, more expensive).
    High,
    /// Urgent (fastest, most expensive).
    Urgent,
}

impl PriorityLevel {
    /// Returns the compute unit price multiplier.
    #[must_use]
    pub fn price_multiplier(&self) -> u64 {
        match self {
            Self::Low => 1,
            Self::Medium => 10,
            Self::High => 100,
            Self::Urgent => 1000,
        }
    }
}

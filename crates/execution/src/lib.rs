//! Live execution engine and transaction management.
//!
//! This crate provides functionality for live position management:
//! - Position monitoring with real-time PnL tracking
//! - Alert system with configurable rules
//! - Wallet management for transaction signing
//! - Transaction building and lifecycle management
//! - Automated strategy execution

/// Prelude module for convenient imports.
pub mod prelude;

/// Alert system.
pub mod alerts;
/// Position monitoring.
pub mod monitor;
/// Strategy execution.
pub mod strategy;
/// Transaction building and sending.
pub mod transaction;
/// Wallet management.
pub mod wallet;

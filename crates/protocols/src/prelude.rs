//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types from the crate.
//!
//! # Example
//!
//! ```rust
//! use clmm_lp_protocols::prelude::*;
//! ```

// Traits
pub use crate::PoolFetcher;

// Orca
pub use crate::orca::provider::OrcaPoolProvider;
pub use crate::orca::whirlpool::{Whirlpool, WhirlpoolParser};

// Solana client
pub use crate::solana_client::SolanaRpcAdapter;

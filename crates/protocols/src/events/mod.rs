//! Event fetching and parsing for CLMM protocols.
//!
//! This module provides functionality to fetch and parse on-chain events
//! from CLMM protocol transactions.

mod fetcher;
mod parser;
mod types;

pub use fetcher::*;
pub use parser::*;
pub use types::*;

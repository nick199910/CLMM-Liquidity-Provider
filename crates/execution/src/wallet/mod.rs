//! Wallet management for transaction signing.
//!
//! Provides secure wallet handling including:
//! - Keypair loading from files
//! - Environment variable support
//! - Memory safety with zeroize

mod keypair;
mod manager;

pub use keypair::Wallet;
pub use manager::WalletManager;

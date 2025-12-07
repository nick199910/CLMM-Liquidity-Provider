//! Wallet management for transaction signing.
//!
//! Provides secure wallet handling including:
//! - Keypair loading from files
//! - Environment variable support
//! - Memory safety with zeroize

use anyhow::{Context, Result};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use std::fs;
use std::path::Path;
use tracing::info;
use zeroize::Zeroizing;

/// Wallet abstraction for signing transactions.
pub struct Wallet {
    /// The keypair.
    keypair: Keypair,
    /// Wallet label.
    label: String,
}

impl Wallet {
    /// Creates a wallet from a keypair.
    pub fn from_keypair(keypair: Keypair, label: impl Into<String>) -> Self {
        Self {
            keypair,
            label: label.into(),
        }
    }

    /// Loads a wallet from a JSON file.
    pub fn from_file(path: impl AsRef<Path>, label: impl Into<String>) -> Result<Self> {
        let path = path.as_ref();
        let label = label.into();

        info!(path = %path.display(), label = %label, "Loading wallet from file");

        let contents =
            Zeroizing::new(fs::read_to_string(path).context("Failed to read keypair file")?);

        let bytes: Vec<u8> =
            serde_json::from_str(&contents).context("Failed to parse keypair JSON")?;

        let bytes_array: [u8; 32] = bytes[..32].try_into().context("Invalid keypair length")?;
        let keypair = Keypair::new_from_array(bytes_array);

        Ok(Self { keypair, label })
    }

    /// Loads a wallet from an environment variable.
    pub fn from_env(var_name: &str, label: impl Into<String>) -> Result<Self> {
        let label = label.into();

        info!(var = var_name, label = %label, "Loading wallet from environment");

        let value = Zeroizing::new(
            std::env::var(var_name)
                .context(format!("Environment variable {} not set", var_name))?,
        );

        // Try to parse as JSON array first
        if let Ok(bytes) = serde_json::from_str::<Vec<u8>>(&value) {
            let bytes_array: [u8; 32] = bytes[..32].try_into().context("Invalid keypair length")?;
            let keypair = Keypair::new_from_array(bytes_array);
            return Ok(Self { keypair, label });
        }

        // Try to parse as base58
        // Try to decode as base58 and use from_base58_string
        let keypair = Keypair::from_base58_string(&value);

        Ok(Self { keypair, label })
    }

    /// Returns the public key.
    #[must_use]
    pub fn pubkey(&self) -> Pubkey {
        self.keypair.pubkey()
    }

    /// Returns the wallet label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Signs a message.
    pub fn sign(&self, message: &[u8]) -> solana_sdk::signature::Signature {
        self.keypair.sign_message(message)
    }

    /// Returns a reference to the keypair for signing transactions.
    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }
}

/// Wallet manager for handling multiple wallets.
pub struct WalletManager {
    /// Available wallets.
    wallets: std::collections::HashMap<String, Wallet>,
    /// Default wallet label.
    default_wallet: Option<String>,
}

impl WalletManager {
    /// Creates a new wallet manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            wallets: std::collections::HashMap::new(),
            default_wallet: None,
        }
    }

    /// Adds a wallet.
    pub fn add_wallet(&mut self, wallet: Wallet) {
        let label = wallet.label.clone();
        self.wallets.insert(label.clone(), wallet);

        if self.default_wallet.is_none() {
            self.default_wallet = Some(label);
        }
    }

    /// Gets a wallet by label.
    pub fn get_wallet(&self, label: &str) -> Option<&Wallet> {
        self.wallets.get(label)
    }

    /// Gets the default wallet.
    pub fn get_default(&self) -> Option<&Wallet> {
        self.default_wallet
            .as_ref()
            .and_then(|label| self.wallets.get(label))
    }

    /// Sets the default wallet.
    pub fn set_default(&mut self, label: &str) -> bool {
        if self.wallets.contains_key(label) {
            self.default_wallet = Some(label.to_string());
            true
        } else {
            false
        }
    }

    /// Lists all wallet labels.
    pub fn list_wallets(&self) -> Vec<&str> {
        self.wallets.keys().map(String::as_str).collect()
    }
}

impl Default for WalletManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_manager() {
        let mut manager = WalletManager::new();

        let keypair = Keypair::new();
        let wallet = Wallet::from_keypair(keypair, "test");

        manager.add_wallet(wallet);

        assert!(manager.get_wallet("test").is_some());
        assert!(manager.get_default().is_some());
    }
}
